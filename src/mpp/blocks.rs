//! Bloques de bajo nivel del formato MPP14: Props, VarMeta/Var2Data, FixedMeta/FixedData
//! y el mapa de campos (donde vive cada campo dentro de esos bloques).

use super::fields::*;
use super::util::*;
use crate::fasthash::FastMap;
use crate::model::{Fields, Unit, Value};

const MAGIC: u32 = 0xFADF_ADBA;

// ---------------------------------------------------------------------------------------------
// Props: diccionario clave -> bytes
// ---------------------------------------------------------------------------------------------

pub struct Props {
    map: FastMap<u32, Vec<u8>>,
}

impl Props {
    pub fn parse(d: &[u8]) -> Props {
        let mut map = FastMap::default();
        let count = u16_at(d, 12) as usize;
        let mut pos = 16usize;
        let mut found = 0usize;
        while found < count {
            if pos + 12 > d.len() {
                break;
            }
            let size = i32_at(d, pos);
            let key = u32_at(d, pos + 4);
            pos += 12;
            if size < 1 || pos + size as usize > d.len() {
                break;
            }
            let size = size as usize;
            map.insert(key, d[pos..pos + size].to_vec());
            pos += size;
            if size % 2 != 0 {
                pos += 1;
            }
            found += 1;
        }
        Props { map }
    }

    pub fn empty() -> Props {
        Props { map: FastMap::default() }
    }

    pub fn bytes(&self, key: u32) -> Option<&[u8]> {
        self.map.get(&key).map(|v| v.as_slice())
    }

    pub fn u8(&self, key: u32) -> u8 {
        self.bytes(key).map_or(0, |b| u8_at(b, 0))
    }

    pub fn u16(&self, key: u32) -> u16 {
        self.bytes(key).map_or(0, |b| u16_at(b, 0))
    }

    pub fn i32(&self, key: u32) -> i32 {
        self.bytes(key).map_or(0, |b| i32_at(b, 0))
    }

    pub fn timestamp(&self, key: u32) -> Option<i64> {
        self.bytes(key).and_then(|b| timestamp(b, 0))
    }

    pub fn text(&self, key: u32) -> Option<String> {
        self.bytes(key).map(|b| utf16(b, 0)).filter(|s| !s.is_empty())
    }
}

// ---------------------------------------------------------------------------------------------
// VarMeta + Var2Data: datos de longitud variable, indexados por (id unico, tipo)
// ---------------------------------------------------------------------------------------------

pub struct VarData {
    index: FastMap<(i32, u16), usize>,
    uids: Vec<i32>,
    data: Vec<u8>,
}

impl VarData {
    pub fn parse(meta: &[u8], data: Vec<u8>) -> Result<VarData, String> {
        let magic = u32_at(meta, 0);
        if magic != 0 && magic != MAGIC {
            return Err("Bloque VarMeta invalido.".into());
        }
        let count = u32_at(meta, 8) as usize;
        let mut index = FastMap::default();
        let mut uids = Vec::new();
        let mut pos = 24usize;
        for _ in 0..count {
            if pos + 12 > meta.len() {
                break;
            }
            let uid = i32_at(meta, pos);
            let offset = u32_at(meta, pos + 4) as usize;
            let kind = u16_at(meta, pos + 8);
            index.insert((uid, kind), offset);
            uids.push(uid);
            pos += 12;
        }
        uids.sort_unstable();
        uids.dedup();
        Ok(VarData { index, uids, data })
    }

    pub fn empty() -> VarData {
        VarData { index: FastMap::default(), uids: Vec::new(), data: Vec::new() }
    }

    /// Identificadores unicos presentes, en orden ascendente.
    pub fn uids(&self) -> &[i32] {
        &self.uids
    }

    pub fn has_uid(&self, uid: i32) -> bool {
        self.uids.binary_search(&uid).is_ok()
    }

    pub fn get(&self, uid: i32, kind: u16) -> Option<&[u8]> {
        let off = *self.index.get(&(uid, kind))?;
        if off.checked_add(4)? > self.data.len() {
            return None;
        }
        let size = i32_at(&self.data, off);
        if size < 0 {
            return None;
        }
        self.data.get(off + 4..(off + 4).checked_add(size as usize)?)
    }

    pub fn text(&self, uid: i32, kind: u16) -> Option<String> {
        self.get(uid, kind).map(|b| utf16(b, 0))
    }
}

// ---------------------------------------------------------------------------------------------
// FixedMeta + FixedData: registros de tamano fijo
// ---------------------------------------------------------------------------------------------

pub struct FixedMeta {
    data: Vec<u8>,
    item_size: usize,
    /// Numero de elementos declarado en la cabecera.
    pub item_count: usize,
    /// Numero de elementos segun el tamano real del bloque.
    pub adjusted_count: usize,
}

impl FixedMeta {
    pub fn parse(data: Vec<u8>, item_size: usize) -> Result<FixedMeta, String> {
        if data.len() < 16 || u32_at(&data, 0) != MAGIC {
            return Err("Bloque FixedMeta invalido.".into());
        }
        let item_count = u32_at(&data, 8) as usize;
        let adjusted_count = if item_size == 0 { 0 } else { (data.len() - 16) / item_size };
        Ok(FixedMeta { data, item_size, item_count, adjusted_count })
    }

    /// Variante para "Fixed2Meta": el tamano del registro depende de la version de Project,
    /// asi que se elige el candidato que cuadra con el otro bloque.
    pub fn parse_guess(data: Vec<u8>, other_count: usize, sizes: &[usize]) -> Result<FixedMeta, String> {
        if data.len() < 16 || u32_at(&data, 0) != MAGIC {
            return Err("Bloque Fixed2Meta invalido.".into());
        }
        let item_count = u32_at(&data, 8) as i64;
        let available = (data.len() - 16) as i64;
        let mut item_size = sizes[0];
        let mut distance = i64::MIN;
        for &test in sizes {
            let t = test as i64;
            if available % t == 0 {
                if (available / t) as usize == other_count {
                    item_size = test;
                    break;
                }
                let test_distance = item_count * t - available;
                if test_distance <= 0 && test_distance > distance {
                    item_size = test;
                    distance = test_distance;
                }
            }
        }
        FixedMeta::parse(data, item_size)
    }

    pub fn item(&self, i: usize) -> Option<&[u8]> {
        if i >= self.adjusted_count {
            return None;
        }
        let s = 16 + i * self.item_size;
        self.data.get(s..s + self.item_size)
    }

    pub fn item_size(&self) -> usize {
        self.item_size
    }
}

pub struct FixedData {
    buf: Vec<u8>,
    /// (desplazamiento, longitud) de cada elemento.
    items: Vec<Option<(usize, usize)>>,
}

impl FixedData {
    /// Elementos ubicados segun los desplazamientos de FixedMeta; el tamano de cada uno es la
    /// distancia al siguiente, limitado por `max_expected` (0 = sin limite).
    pub fn with_meta(meta: &FixedMeta, buf: Vec<u8>, max_expected: usize) -> FixedData {
        let n = meta.adjusted_count;
        let len = buf.len() as i64;
        let mut items = vec![None; n];
        for i in 0..n {
            let off = meta.item(i).map_or(-1, |m| i32_at(m, 4) as i64);
            if off < 0 || off > len {
                continue;
            }
            let mut size = if i + 1 == n {
                len - off
            } else {
                meta.item(i + 1).map_or(0, |m| i32_at(m, 4) as i64) - off
            };
            let available = len - off;
            if size < 0 || size > available {
                size = if max_expected == 0 { available } else { (max_expected as i64).min(available) };
            }
            if max_expected != 0 && size > max_expected as i64 {
                size = max_expected as i64;
            }
            if size > 0 {
                items[i] = Some((off as usize, size as usize));
            }
        }
        FixedData { buf, items }
    }

    /// Elementos ubicados segun FixedMeta, todos del mismo tamano.
    pub fn with_meta_sized(meta: &FixedMeta, buf: Vec<u8>, item_size: usize) -> FixedData {
        let n = meta.adjusted_count;
        let len = buf.len() as i64;
        let mut items = vec![None; n];
        for (i, slot) in items.iter_mut().enumerate() {
            let off = meta.item(i).map_or(-1, |m| i32_at(m, 4) as i64);
            if off < 0 || off > len {
                continue;
            }
            let size = (item_size as i64).min(len - off);
            *slot = Some((off as usize, size as usize));
        }
        FixedData { buf, items }
    }

    /// Elementos consecutivos de tamano fijo.
    pub fn sequential(buf: Vec<u8>, item_size: usize) -> FixedData {
        let n = if item_size == 0 { 0 } else { buf.len() / item_size };
        let items = (0..n).map(|i| Some((i * item_size, item_size))).collect();
        FixedData { buf, items }
    }

    pub fn empty() -> FixedData {
        FixedData { buf: Vec::new(), items: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn get(&self, i: usize) -> Option<&[u8]> {
        let (off, size) = (*self.items.get(i)?)?;
        self.buf.get(off..off + size)
    }

    pub fn index_from_offset(&self, offset: usize) -> Option<usize> {
        self.items.iter().position(|it| matches!(it, Some((o, _)) if *o == offset))
    }
}

// ---------------------------------------------------------------------------------------------
// Mapa de campos
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Loc {
    Fixed,
    Var,
    Meta,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub struct FieldItem {
    pub loc: Loc,
    pub block: usize,
    pub offset: usize,
    pub var_key: u16,
    pub dt: u8,
}

/// Valores de listas de busqueda (campos personalizados con tabla de valores).
pub type Lookup = FastMap<i32, Vec<u8>>;

pub struct FieldMap {
    pub table: &'static FieldTable,
    pub items: FastMap<u16, FieldItem>,
    pub max_fixed: [usize; 2],
    /// Campos que se leen de cada elemento: los que tienen nombre y un lugar en el archivo.
    order: Vec<u16>,
}

fn fixed_size(dt: u8) -> usize {
    match dt {
        DT_DATE | DT_INTEGER | DT_DURATION => 4,
        DT_TIME_UNITS | DT_CONSTRAINT | DT_PRIORITY | DT_PERCENTAGE | DT_TASK_TYPE | DT_ACCRUE | DT_SHORT | DT_BOOLEAN
        | DT_DELAY | DT_WORKGROUP | DT_RATE_UNITS | DT_EARNED_VALUE_METHOD | DT_RESOURCE_REQUEST_TYPE => 2,
        DT_CURRENCY | DT_UNITS | DT_RATE | DT_WORK => 8,
        DT_WORK_UNITS => 1,
        DT_GUID => 16,
        _ => 0,
    }
}

impl FieldMap {
    /// Decodifica el mapa de campos guardado en las propiedades del proyecto
    /// (registros de 28 bytes: mascara, desplazamiento, id de campo, categoria).
    pub fn parse(data: &[u8], table: &'static FieldTable) -> FieldMap {
        let mut fm = FieldMap { table, items: FastMap::default(), max_fixed: [0, 0], order: Vec::new() };
        let mut index = 0usize;
        let mut last_offset = 0usize;
        let mut block = 0usize;
        while index + 28 <= data.len() {
            let block_offset = u16_at(data, index + 4) as usize;
            let type_value = u32_at(data, index + 12);
            let category = u16_at(data, index + 20);
            let var_key = (type_value & 0xFFFF) as u16;
            let field_index = var_key as usize;
            let known = (type_value & 0xFFFF_0000) == table.base && field_index < table.dt.len();
            let dt = if known { table.dt[field_index] } else { DT_NONE };
            let loc = match category {
                0x0B | 0x64 => Loc::Meta,
                _ => {
                    if block_offset != 65535 {
                        if block_offset < last_offset {
                            block += 1;
                        }
                        last_offset = block_offset;
                        if block < 2 {
                            let end = block_offset + fixed_size(dt);
                            if end > fm.max_fixed[block] {
                                fm.max_fixed[block] = end;
                            }
                        }
                        Loc::Fixed
                    } else if var_key != 0 {
                        Loc::Var
                    } else {
                        Loc::Unknown
                    }
                }
            };
            if known && dt != DT_NONE {
                let canon = table.canon[field_index];
                fm.items.insert(canon, FieldItem { loc, block, offset: block_offset, var_key, dt });
            }
            index += 28;
        }
        // El mapa trae cientos de campos internos o de Project Server que no se muestran en
        // ninguna salida; leerlos para cada tarea seria trabajo perdido.
        let mut order: Vec<u16> = fm
            .items
            .iter()
            .filter(|(id, it)| matches!(it.loc, Loc::Fixed | Loc::Var) && table.names.binary_search_by_key(*id, |(n, _)| *n).is_ok())
            .map(|(id, _)| *id)
            .collect();
        order.sort_unstable();
        fm.order = order;
        fm
    }

    /// Lee todos los campos con nombre del elemento `uid`.
    pub fn read_all(&self, uid: i32, fixed: &[Option<&[u8]>; 2], var: &VarData, sc: &Scale, lookup: &Lookup) -> Fields {
        let mut fields = Fields::default();
        for &canon in &self.order {
            if let Some(v) = self.read(canon, uid, fixed, var, sc, lookup) {
                fields.insert(canon, v);
            }
        }
        fields
    }

    pub fn fixed_offset(&self, canon: u16) -> Option<usize> {
        self.items.get(&canon).filter(|i| i.loc == Loc::Fixed).map(|i| i.offset)
    }

    pub fn var_key(&self, canon: u16) -> Option<u16> {
        self.items.get(&canon).map(|i| i.var_key)
    }

    fn units_field(&self, canon: u16) -> Option<u16> {
        self.table.units.iter().find(|(d, _)| *d == canon).map(|(_, u)| *u)
    }

    fn read_units(&self, canon: u16, uid: i32, fixed: &[Option<&[u8]>; 2], var: &VarData, sc: &Scale) -> Option<Unit> {
        let it = self.items.get(&self.units_field(canon)?)?;
        match it.loc {
            Loc::Fixed => {
                let d = fixed.get(it.block).copied().flatten()?;
                if it.offset >= d.len() {
                    return None;
                }
                Some(duration_units(u16_at(d, it.offset), Some(sc.default_units)))
            }
            Loc::Var => {
                // Sin dato, MPXJ lee un cero, que equivale a "dias".
                let code = var.get(uid, it.var_key).map_or(0, |b| u16_at(b, 0));
                Some(duration_units(code, Some(sc.default_units)))
            }
            _ => None,
        }
    }

    /// Lee el valor de un campo para el elemento `uid`.
    pub fn read(&self, canon: u16, uid: i32, fixed: &[Option<&[u8]>; 2], var: &VarData, sc: &Scale, lookup: &Lookup) -> Option<Value> {
        let it = self.items.get(&canon)?;
        match it.loc {
            Loc::Fixed => {
                let d = fixed.get(it.block).copied().flatten()?;
                let o = it.offset;
                if o >= d.len() {
                    return None;
                }
                match it.dt {
                    DT_DATE => timestamp(d, o).map(Value::Date),
                    DT_INTEGER => Some(Value::Int(i32_at(d, o))),
                    DT_DURATION => {
                        let units = self.read_units(canon, uid, fixed, var, sc).unwrap_or(sc.default_units);
                        adjusted_duration(sc, i32_at(d, o), units).map(Value::Dur)
                    }
                    DT_TIME_UNITS => Some(Value::Unit(duration_units(u16_at(d, o), Some(sc.default_units)))),
                    DT_PERCENTAGE => {
                        let v = u16_at(d, o);
                        if v <= 100 {
                            Some(Value::Num(v as f64))
                        } else {
                            None
                        }
                    }
                    DT_CURRENCY | DT_UNITS => {
                        let a = f64_at(d, o);
                        Some(Value::Num(if a.abs() < 0.1 { 0.0 } else { a } / 100.0))
                    }
                    DT_RATE => Some(Value::Num(f64_at(d, o))),
                    DT_WORK => {
                        let w = f64_at(d, o);
                        Some(Value::Dur(crate::model::Dur { value: if w.abs() < 1000.0 { 0.0 } else { w } / 60000.0, unit: Unit::Hours }))
                    }
                    DT_BOOLEAN => Some(Value::Bool(u16_at(d, o) != 0)),
                    DT_DELAY => Some(Value::Dur(crate::model::Dur { value: u16_at(d, o) as f64 / 600.0, unit: Unit::Hours })),
                    DT_WORK_UNITS => Some(Value::Int(u8_at(d, o) as i32)),
                    DT_SHORT | DT_CONSTRAINT | DT_PRIORITY | DT_TASK_TYPE | DT_ACCRUE | DT_WORKGROUP | DT_RATE_UNITS
                    | DT_EARNED_VALUE_METHOD | DT_RESOURCE_REQUEST_TYPE => Some(Value::Int(u16_at(d, o) as i32)),
                    _ => None,
                }
            }
            Loc::Var => {
                let raw = var.get(uid, it.var_key);
                match it.dt {
                    DT_STRING => {
                        let b = raw?;
                        match lookup_ref(b) {
                            Some(id) => lookup.get(&id).map(|v| Value::Text(utf16(v, 0))),
                            None => Some(Value::Text(utf16(b, 0))),
                        }
                    }
                    DT_NUMERIC => {
                        let b = raw?;
                        match lookup_ref(b) {
                            Some(id) => lookup.get(&id).filter(|v| v.len() >= 8).map(|v| Value::Num(f64_at(v, 0))),
                            None if b.len() >= 8 => Some(Value::Num(f64_at(b, 0))),
                            None => None,
                        }
                    }
                    DT_DATE => {
                        let b = raw?;
                        let b = match lookup_ref(b) {
                            Some(id) => lookup.get(&id)?.as_slice(),
                            None => b,
                        };
                        if b.len() == 512 {
                            Some(Value::Text(utf16(b, 0)))
                        } else {
                            timestamp(b, 0).map(Value::Date)
                        }
                    }
                    DT_DURATION => {
                        let b = raw?;
                        if b.len() == 512 {
                            Some(Value::Text(utf16(b, 0)))
                        } else if b.len() >= 4 {
                            let units = self.read_units(canon, uid, fixed, var, sc).unwrap_or(Unit::Hours);
                            adjusted_duration(sc, i32_at(b, 0), units).map(Value::Dur)
                        } else {
                            None
                        }
                    }
                    DT_TIME_UNITS => Some(Value::Unit(duration_units(raw.map_or(0, |b| u16_at(b, 0)), Some(sc.default_units)))),
                    DT_CURRENCY => {
                        let b = raw.filter(|b| b.len() >= 8)?;
                        let a = f64_at(b, 0);
                        Some(Value::Num(if a.abs() < 0.1 { 0.0 } else { a } / 100.0))
                    }
                    DT_INTEGER => raw.filter(|b| b.len() >= 4).map(|b| Value::Int(i32_at(b, 0))),
                    DT_WORK => {
                        let b = raw.filter(|b| b.len() >= 8)?;
                        let w = f64_at(b, 0);
                        Some(Value::Dur(crate::model::Dur { value: if w.abs() < 1000.0 { 0.0 } else { w } / 60000.0, unit: Unit::Hours }))
                    }
                    DT_NOTES => {
                        let text = rtf_to_text(&latin1(raw?, 0));
                        if text.is_empty() {
                            None
                        } else {
                            Some(Value::Text(text))
                        }
                    }
                    DT_BOOLEAN => raw.map(|b| Value::Bool(u16_at(b, 0) != 0)),
                    DT_SHORT | DT_PERCENTAGE | DT_ACCRUE | DT_WORKGROUP | DT_RATE_UNITS | DT_EARNED_VALUE_METHOD
                    | DT_RESOURCE_REQUEST_TYPE | DT_BOOKING_TYPE => raw.filter(|b| b.len() >= 2).map(|b| Value::Int(u16_at(b, 0) as i32)),
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// Si el valor es una referencia a una lista de valores, devuelve el id del elemento.
fn lookup_ref(b: &[u8]) -> Option<i32> {
    let flag = u16_at(b, 0);
    if (flag == 0x0701 || flag == 0x0401) && b.len() >= 6 && b.len() <= 26 {
        Some(i32_at(b, 2))
    } else {
        None
    }
}
