//! Modelo en memoria del proyecto leido desde el .mpp.

use crate::fasthash::FastMap;
use std::collections::HashMap;

/// Unidades de duracion de MS Project.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Minutes,
    Hours,
    Days,
    Weeks,
    Months,
    Percent,
    ElapsedMinutes,
    ElapsedHours,
    ElapsedDays,
    ElapsedWeeks,
    ElapsedMonths,
    ElapsedPercent,
}

impl Unit {
    /// Sufijo corto, igual al que usa MS Project en ingles ("h", "d", "ed"...).
    pub fn suffix(self) -> &'static str {
        match self {
            Unit::Minutes => "m",
            Unit::Hours => "h",
            Unit::Days => "d",
            Unit::Weeks => "w",
            Unit::Months => "mo",
            Unit::Percent => "%",
            Unit::ElapsedMinutes => "em",
            Unit::ElapsedHours => "eh",
            Unit::ElapsedDays => "ed",
            Unit::ElapsedWeeks => "ew",
            Unit::ElapsedMonths => "emo",
            Unit::ElapsedPercent => "e%",
        }
    }
}

/// Duracion: valor en las unidades indicadas.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Dur {
    pub value: f64,
    pub unit: Unit,
}

/// Valor de un campo. Las fechas son segundos desde el 1983-12-31 00:00 (epoca de MS Project).
#[derive(Clone, PartialEq, Debug)]
pub enum Value {
    Text(String),
    Int(i32),
    Num(f64),
    Date(i64),
    Dur(Dur),
    Bool(bool),
    Unit(Unit),
}

/// Campos indexados por el identificador canonico de MS Project (ver `mpp::fields`).
pub type Fields = FastMap<u16, Value>;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RelType {
    FinishFinish,
    FinishStart,
    StartFinish,
    StartStart,
}

impl RelType {
    pub fn code(self) -> &'static str {
        match self {
            RelType::FinishFinish => "FF",
            RelType::FinishStart => "FS",
            RelType::StartFinish => "SF",
            RelType::StartStart => "SS",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Relation {
    pub uid: i32,
    pub pred_uid: i32,
    pub succ_uid: i32,
    pub kind: RelType,
    pub lag: Option<Dur>,
}

#[derive(Clone, Debug, Default)]
pub struct Task {
    pub id: i32,
    pub uid: i32,
    pub name: Option<String>,
    pub outline_level: i32,
    pub outline_number: String,
    /// Codigo EDT (WBS); si el archivo no define uno propio, coincide con el numero de esquema.
    pub wbs: String,
    pub parent_uid: Option<i32>,
    pub start: Option<i64>,
    pub finish: Option<i64>,
    pub duration: Option<Dur>,
    pub total_slack: Option<Dur>,
    pub summary: bool,
    pub milestone: bool,
    pub critical: bool,
    pub active: bool,
    pub manual: bool,
    pub notes: Option<String>,
    /// Flag1..Flag20.
    pub flags: [bool; 20],
    pub fields: Fields,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResourceKind {
    Work,
    Material,
    Cost,
}

#[derive(Clone, Debug)]
pub struct Resource {
    pub id: i32,
    pub uid: i32,
    pub name: Option<String>,
    pub kind: ResourceKind,
    pub calendar_uid: Option<i32>,
    pub base_calendar: Option<String>,
    pub start: Option<i64>,
    pub finish: Option<i64>,
    pub notes: Option<String>,
    pub flags: [bool; 20],
    pub fields: Fields,
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub uid: i32,
    pub task_uid: i32,
    pub resource_uid: Option<i32>,
    /// 1.0 = 100 %.
    pub units: Option<f64>,
    pub work: Option<Dur>,
    pub start: Option<i64>,
    pub finish: Option<i64>,
    pub fields: Fields,
}

/// Horario de un dia: `None` = hereda del calendario base; lista vacia = no laborable.
/// Cada tramo es (inicio, fin) en minutos desde medianoche.
pub type DayHours = Option<Vec<(u32, u32)>>;

#[derive(Clone, Debug)]
pub struct Calendar {
    pub uid: i32,
    pub name: Option<String>,
    pub base_uid: Option<i32>,
    pub resource_uid: Option<i32>,
    /// Domingo a sabado.
    pub days: [DayHours; 7],
}

#[derive(Clone, Debug)]
pub struct Column {
    /// Identificador completo del campo (clase en los 16 bits altos).
    pub field: u32,
    pub title: Option<String>,
    pub width: u8,
}

#[derive(Clone, Debug)]
pub struct Table {
    pub name: String,
    pub resource: bool,
    pub columns: Vec<Column>,
}

#[derive(Clone, Debug)]
pub struct ProjectProps {
    pub title: Option<String>,
    pub author: Option<String>,
    pub last_author: Option<String>,
    pub company: Option<String>,
    pub manager: Option<String>,
    pub start: Option<i64>,
    pub finish: Option<i64>,
    pub status_date: Option<i64>,
    pub minutes_per_day: i32,
    pub minutes_per_week: i32,
    pub days_per_month: i32,
    pub default_units: Unit,
    pub default_calendar: Option<String>,
    pub critical_slack_limit_days: i32,
    pub currency_symbol: Option<String>,
    pub app_name: String,
    pub app_version: i32,
}

#[derive(Clone, Debug)]
pub struct Project {
    pub props: ProjectProps,
    pub tasks: Vec<Task>,
    pub resources: Vec<Resource>,
    pub assignments: Vec<Assignment>,
    pub relations: Vec<Relation>,
    pub calendars: Vec<Calendar>,
    pub tables: Vec<Table>,
    /// Alias (nombre visible) de campos personalizados de tarea, por id canonico.
    pub task_aliases: HashMap<u16, String>,
    /// Alias de campos personalizados de recurso, por id canonico.
    pub resource_aliases: HashMap<u16, String>,
}

impl Project {
    pub fn task_by_uid(&self, uid: i32) -> Option<&Task> {
        self.tasks.iter().find(|t| t.uid == uid)
    }

    pub fn resource_by_uid(&self, uid: i32) -> Option<&Resource> {
        self.resources.iter().find(|r| r.uid == uid)
    }
}

// ---------------------------------------------------------------------------------------------
// Fechas
// ---------------------------------------------------------------------------------------------

/// Dias entre 1970-01-01 y la epoca de MS Project (1983-12-31).
const EPOCH_UNIX_DAYS: i64 = 5112;

/// Descompone una fecha (segundos desde la epoca de Project) en (anio, mes, dia, hora, min, seg).
pub fn civil(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86400) + EPOCH_UNIX_DAYS;
    let rem = secs.rem_euclid(86400);
    // Algoritmo "civil_from_days" de Howard Hinnant.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d, (rem / 3600) as u32, ((rem % 3600) / 60) as u32, (rem % 60) as u32)
}

/// Fecha en ISO 8601 sin zona horaria (MS Project guarda hora local): `2026-05-20T02:00:00`.
pub fn iso(secs: i64) -> String {
    let (y, m, d, h, mi, s) = civil(secs);
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}", y, m, d, h, mi, s)
}

/// Numero de serie de Excel (sistema 1900) para una fecha.
pub fn excel_serial(secs: i64) -> f64 {
    30681.0 + secs as f64 / 86400.0
}

/// Dia de la semana: 0 = domingo ... 6 = sabado.
pub fn weekday(secs: i64) -> u32 {
    // 1983-12-31 fue sabado.
    ((secs.div_euclid(86400) + 6).rem_euclid(7)) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fechas() {
        assert_eq!(iso(0), "1983-12-31T00:00:00");
        assert_eq!(weekday(0), 6);
        // 2026-05-20 02:00 (miercoles): numero de serie de Excel 46162 + 2/24.
        let t = (46162 - 30681) * 86400 + 7200;
        assert_eq!(iso(t), "2026-05-20T02:00:00");
        assert_eq!(weekday(t), 3);
        assert!((excel_serial(t) - (46162.0 + 2.0 / 24.0)).abs() < 1e-9);
        // Anio bisiesto.
        assert_eq!(iso((45351 - 30681) * 86400), "2024-02-29T00:00:00");
    }
}
