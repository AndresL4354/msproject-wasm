// Cargador de doc2json.wasm para el navegador y Node. Sin dependencias.
//
//   await init('/doc2json.wasm');          // una sola vez (en Node: init(bytes del .wasm))
//   const doc = await docToJson(archivo);  // { type, metadata, content, text, ... }
//
// Acepta .docx, .pdf, .pptx y .xlsx; el formato se reconoce por el contenido. La conversion ocurre donde
// corre este codigo: el archivo no se envia a ningun servidor.

let modulo; // Promise<WebAssembly.Module>
let instancia; // Promise con los exports de la instancia en uso

async function compilar(origen) {
  if (origen instanceof WebAssembly.Module) return origen;
  if (origen instanceof ArrayBuffer || ArrayBuffer.isView(origen)) return WebAssembly.compile(origen);
  const respuesta = await (typeof origen === 'string' || origen instanceof URL ? fetch(origen) : origen);
  if (!respuesta.ok) throw new Error(`No se pudo descargar doc2json.wasm (HTTP ${respuesta.status}).`);
  // compileStreaming exige el tipo application/wasm; si el servidor no lo envia, se compila desde los bytes.
  const tipo = respuesta.headers.get('content-type') || '';
  if (WebAssembly.compileStreaming && tipo.startsWith('application/wasm')) return WebAssembly.compileStreaming(respuesta);
  return WebAssembly.compile(await respuesta.arrayBuffer());
}

/** Descarga y compila el modulo. Llamarla varias veces no lo descarga de nuevo. */
export function init(origen) {
  if (!modulo) {
    if (origen == null) return Promise.reject(new TypeError('init() necesita la URL o los bytes de doc2json.wasm.'));
    const intento = compilar(origen);
    modulo = intento;
    // Si la descarga falla, un init() posterior puede reintentar.
    intento.catch(() => {
      if (modulo === intento) modulo = undefined;
    });
  }
  return modulo.then(() => undefined);
}

/** Limites de una conversion: [MB descomprimidos, paginas, caracteres]; 0 = valor por defecto. */
function limites(opciones) {
  const l = (opciones && opciones.limits) || {};
  const entero = (v, nombre) => {
    if (v == null) return 0;
    const n = Math.floor(v);
    if (!(n >= 1 && n <= 0xffffffff)) throw new RangeError(`limits.${nombre} debe ser un entero positivo.`);
    return n;
  };
  return [entero(l.maxDecompressedMB, 'maxDecompressedMB'), entero(l.maxPages, 'maxPages'), entero(l.maxChars, 'maxChars')];
}

async function convertir(archivo, modo, opciones) {
  if (!modulo) throw new Error('Llame primero a init() con la ubicacion de doc2json.wasm.');
  const lim = limites(opciones);
  if (typeof Blob !== 'undefined' && archivo instanceof Blob) archivo = await archivo.arrayBuffer();
  const entrada = archivo instanceof Uint8Array ? archivo : new Uint8Array(archivo);
  if (!instancia) instancia = modulo.then((m) => WebAssembly.instantiate(m, {})).then((i) => i.exports);
  const usada = instancia;
  const x = await usada;
  // Un doc2json.wasm anterior a los limites recibe solo tres argumentos y los ignoraria en silencio.
  if (lim.some((v) => v !== 0) && x.doc_convert.length < 6) {
    throw new Error('Este doc2json.wasm no admite limites de recursos (limits): actualicelo a la version de este cargador.');
  }
  // Desde aqui no hay ningun await: dos conversiones simultaneas no se mezclan dentro del modulo.
  try {
    const ptr = x.doc_alloc(entrada.length);
    if (entrada.length && !ptr) throw new Error('No hay memoria suficiente para un archivo de este tamano.');
    new Uint8Array(x.memory.buffer, ptr, entrada.length).set(entrada);
    // Los limites van como argumentos de cada conversion: no quedan para la siguiente.
    const estado = x.doc_convert(ptr, entrada.length, modo, lim[0], lim[1], lim[2]);
    const salida = new TextDecoder().decode(new Uint8Array(x.memory.buffer, x.doc_result_ptr(), x.doc_result_len()));
    x.doc_free(ptr, entrada.length);
    if (estado !== 0) throw new Error(salida);
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

/** Convierte un .docx, .pdf, .pptx o .xlsx en JSON, como texto. Con `full` agrega estilos, tramos con formato y lineas de PDF. */
export function docToJsonText(archivo, opciones) {
  return convertir(archivo, opciones && opciones.full ? 1 : 0, opciones);
}

/** Convierte un .docx, .pdf, .pptx o .xlsx en un objeto con metadatos, bloques de contenido y texto plano. */
export async function docToJson(archivo, opciones) {
  return JSON.parse(await docToJsonText(archivo, opciones));
}

// Operaciones de doc_convert (byte bajo del modo; el parametro va en los bits 8 a 31):
// 0 JSON, 1 JSON completo, 2 Markdown, 3 fragmentos (tamano maximo), 4 y 5 JSON con imagenes.
const MARKDOWN = 2;
const FRAGMENTOS = 3;

/**
 * Convierte un .docx o un .pdf en Markdown: titulos, listas, tablas GFM y notas al pie. El texto
 * del documento va escapado (no trae HTML ni enlaces que no sean http, https o mailto).
 */
export function docToMarkdown(archivo, opciones) {
  return convertir(archivo, MARKDOWN, opciones);
}

/**
 * Corta un .docx o un .pdf en fragmentos de hasta `maxChars` caracteres (2000 por defecto), con
 * la ruta de titulos de su seccion y las paginas que cubren. Listo para indexar o enviar a un modelo.
 */
export async function docToChunks(archivo, opciones) {
  const valor = opciones == null ? undefined : opciones.maxChars;
  if (valor !== undefined && !(Number.isInteger(valor) && valor >= 1 && valor <= 0xffffff)) {
    throw new RangeError('maxChars debe ser un entero entre 1 y 16777215.');
  }
  // 0 = tamano por defecto. El modo cabe en 32 bits sin signo; WebAssembly lo recibe igual.
  return JSON.parse(await convertir(archivo, FRAGMENTOS + (valor || 0) * 256, opciones));
}
