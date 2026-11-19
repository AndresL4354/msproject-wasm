// Prueba los cargadores y los .wasm de esta carpeta desde Node:
//
//   node web/prueba.mjs                 documentos de ejemplo/ y manejo de errores
//   node web/prueba.mjs plan.mpp        ademas convierte ese .mpp; si existen plan.xlsx y
//                                       plan.json a su lado, exige que la salida sea identica
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import * as doc from './doc2json.js';
import * as mpp from './mpp2xlsx.js';

const aqui = path.dirname(fileURLToPath(import.meta.url));
const ejemplo = path.join(aqui, '..', 'ejemplo');
let fallos = 0;

function comprobar(nombre, ok, detalle = '') {
  if (!ok) fallos++;
  console.log(`${ok ? 'ok   ' : 'FALLA'} ${nombre}${detalle ? ` (${detalle})` : ''}`);
}

async function mensajeDeError(promesa) {
  try {
    await promesa;
    return null;
  } catch (e) {
    return e.message;
  }
}

const leerSiExiste = (ruta) => readFile(ruta).catch(() => null);

comprobar('convertir antes de init() se rechaza', (await mensajeDeError(doc.docToJson(new Uint8Array(4)))) !== null);
await doc.init(await readFile(path.join(aqui, 'doc2json.wasm')));
await mpp.init(await readFile(path.join(aqui, 'mpp2xlsx.wasm')));

for (const nombre of ['procedimiento.docx', 'procedimiento.pdf']) {
  const bytes = await readFile(path.join(ejemplo, nombre));
  const esperado = await readFile(path.join(ejemplo, `${nombre}.json`), 'utf8');
  const texto = await doc.docToJsonText(bytes);
  comprobar(`${nombre}: JSON identico al de ejemplo/`, texto === esperado);
  const objeto = await doc.docToJson(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength));
  comprobar(`${nombre}: objeto desde un ArrayBuffer`, objeto.type === nombre.split('.').pop() && objeto.content.length > 0);
  comprobar(`${nombre}: la salida completa trae mas datos`, (await doc.docToJsonText(bytes, { full: true })).length > texto.length);
}

{
  // Limites de recursos: con pocos caracteres se entrega la primera parte con un aviso, y los
  // limites de una llamada no quedan para la siguiente.
  const bytes = await readFile(path.join(ejemplo, 'procedimiento.docx'));
  const corto = await doc.docToJson(bytes, { limits: { maxChars: 100 } });
  comprobar('limits.maxChars corta el texto y avisa', (corto.warnings || []).some((w) => /limite de 100 caracteres/.test(w)));
  const completo = await doc.docToJson(bytes);
  comprobar('los limites no quedan para la conversion siguiente', !completo.warnings && completo.content.length > corto.content.length);
  comprobar('limits rechaza valores invalidos', /maxPages/.test(await mensajeDeError(doc.docToJson(bytes, { limits: { maxPages: 0 } }))));
  // Un .wasm anterior (doc_convert con tres parametros) no puede ignorar los limites en silencio.
  // Modulo minimo armado a mano: solo exporta doc_convert(i32, i32, i32) -> i32.
  const anterior = await import('./doc2json.js?wasm-anterior');
  await anterior.init(new Uint8Array([
    0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00,
    0x01, 0x08, 0x01, 0x60, 0x03, 0x7f, 0x7f, 0x7f, 0x01, 0x7f,
    0x03, 0x02, 0x01, 0x00,
    0x07, 0x0f, 0x01, 0x0b, ...new TextEncoder().encode('doc_convert'), 0x00, 0x00,
    0x0a, 0x06, 0x01, 0x04, 0x00, 0x41, 0x00, 0x0b,
  ]));
  comprobar('limits con un .wasm anterior da error', /no admite limites/.test(await mensajeDeError(anterior.docToJson(bytes, { limits: { maxChars: 100 } }))));
}

comprobar('doc2json rechaza un archivo que no es documento', /no reconocido/.test(await mensajeDeError(doc.docToJson(new TextEncoder().encode('hola mundo')))));
comprobar('doc2json rechaza un archivo vacio', /vac/.test(await mensajeDeError(doc.docToJson(new Uint8Array(0)))));
comprobar('mpp2xlsx rechaza un archivo que no es .mpp', /no es un \.mpp/.test(await mensajeDeError(mpp.mppToXlsx(await readFile(path.join(ejemplo, 'procedimiento.pdf'))))));

// Contenedor OLE2 cuya cadena DIFAT apunta a si misma: debe dar un error claro, no agotar la memoria.
for (const [desplazamiento, largo] of [[9, 1024], [12, 8192]]) {
  const malo = new Uint8Array(largo);
  malo.set([0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]);
  malo[30] = desplazamiento;
  malo[32] = 6;
  comprobar(`mpp2xlsx rechaza un contenedor malicioso de ${largo} bytes`, /no soportado/.test(await mensajeDeError(mpp.mppToJson(malo))));
}

const plan = process.argv[2];
if (plan) {
  const bytes = await readFile(plan);
  const base = plan.replace(/\.mpp$/i, '');
  const xlsx = await mpp.mppToXlsx(bytes);
  comprobar('mpp -> xlsx es un ZIP', xlsx[0] === 0x50 && xlsx[1] === 0x4b, `${xlsx.length} bytes`);
  const texto = await mpp.mppToJsonText(bytes);
  const datos = JSON.parse(texto);
  comprobar('mpp -> json trae las tareas', datos.tasks.length > 0 && datos.tasks.length === datos.project.task_count, `${datos.tasks.length} tareas`);
  comprobar('mpp -> json completo trae los campos estandar', 'fields' in (await mpp.mppToJson(bytes, { full: true })).tasks[0]);
  const xlsxEsperado = await leerSiExiste(`${base}.xlsx`);
  if (xlsxEsperado) comprobar('mpp -> xlsx identico al guardado', Buffer.compare(Buffer.from(xlsx), xlsxEsperado) === 0);
  const jsonEsperado = await leerSiExiste(`${base}.json`);
  if (jsonEsperado) comprobar('mpp -> json identico al guardado', texto === jsonEsperado.toString('utf8'));
}

console.log(fallos === 0 ? 'Todo bien.' : `Fallos: ${fallos}`);
process.exit(fallos === 0 ? 0 : 1);
