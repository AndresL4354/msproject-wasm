// Cargador de mpp2xlsx.wasm para el navegador y Node. Sin dependencias.
//
//   await init('/mpp2xlsx.wasm');           // una sola vez (en Node: init(bytes del .wasm))
//   const xlsx = await mppToXlsx(archivo);  // Uint8Array con el .xlsx
//   const plan = await mppToJson(archivo);  // { project, tasks, resources, assignments, ... }
//
// La conversion ocurre donde corre este codigo: el archivo no se envia a ningun servidor.

export const XLSX_MIME = 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet';

let modulo; // Promise<WebAssembly.Module>
let instancia; // Promise con los exports de la instancia en uso

async function compilar(origen) {
  if (origen instanceof WebAssembly.Module) return origen;
  if (origen instanceof ArrayBuffer || ArrayBuffer.isView(origen)) return WebAssembly.compile(origen);
  const respuesta = await (typeof origen === 'string' || origen instanceof URL ? fetch(origen) : origen);
  if (!respuesta.ok) throw new Error(`No se pudo descargar mpp2xlsx.wasm (HTTP ${respuesta.status}).`);
  // compileStreaming exige el tipo application/wasm; si el servidor no lo envia, se compila desde los bytes.
  const tipo = respuesta.headers.get('content-type') || '';
  if (WebAssembly.compileStreaming && tipo.startsWith('application/wasm')) return WebAssembly.compileStreaming(respuesta);
  return WebAssembly.compile(await respuesta.arrayBuffer());
}

/** Descarga y compila el modulo. Llamarla varias veces no lo descarga de nuevo. */
export function init(origen) {
  if (!modulo) {
    if (origen == null) return Promise.reject(new TypeError('init() necesita la URL o los bytes de mpp2xlsx.wasm.'));
    const intento = compilar(origen);
    modulo = intento;
    // Si la descarga falla, un init() posterior puede reintentar.
    intento.catch(() => {
      if (modulo === intento) modulo = undefined;
    });
  }
  return modulo.then(() => undefined);
}

async function convertir(archivo, modo) {
  if (!modulo) throw new Error('Llame primero a init() con la ubicacion de mpp2xlsx.wasm.');
  if (typeof Blob !== 'undefined' && archivo instanceof Blob) archivo = await archivo.arrayBuffer();
  const entrada = archivo instanceof Uint8Array ? archivo : new Uint8Array(archivo);
  if (!instancia) instancia = modulo.then((m) => WebAssembly.instantiate(m, {})).then((i) => i.exports);
  const usada = instancia;
  const x = await usada;
  // Desde aqui no hay ningun await: dos conversiones simultaneas no se mezclan dentro del modulo.
  try {
    const ptr = x.mpp_alloc(entrada.length);
    if (entrada.length && !ptr) throw new Error('No hay memoria suficiente para un archivo de este tamano.');
    new Uint8Array(x.memory.buffer, ptr, entrada.length).set(entrada);
    const estado = x.mpp_convert(ptr, entrada.length, modo);
    const salida = new Uint8Array(x.memory.buffer, x.mpp_result_ptr(), x.mpp_result_len()).slice();
    x.mpp_free(ptr, entrada.length);
    if (estado !== 0) throw new Error(new TextDecoder().decode(salida));
    return salida;
  } catch (e) {
    if (e instanceof WebAssembly.RuntimeError) {
      // El modulo aborto (tipicamente por falta de memoria) y esa instancia ya no es confiable:
      // la siguiente conversion crea una nueva.
      if (instancia === usada) instancia = undefined;
      throw new Error('No se pudo convertir el archivo: el conversor se quedo sin memoria o el archivo esta danado.');
    }
    throw e;
  }
}

/** Convierte un .mpp en un libro de Excel (.xlsx). */
export function mppToXlsx(archivo) {
  return convertir(archivo, 0);
}

/** Convierte un .mpp en JSON, como texto. Con `full`, cada tarea trae todos sus campos estandar. */
export async function mppToJsonText(archivo, opciones) {
  return new TextDecoder().decode(await convertir(archivo, opciones && opciones.full ? 2 : 1));
}

/** Convierte un .mpp en un objeto con el proyecto, sus tareas, recursos y asignaciones. */
export async function mppToJson(archivo, opciones) {
  return JSON.parse(await mppToJsonText(archivo, opciones));
}
