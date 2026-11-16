/** De donde sale doc2json.wasm: una URL, una respuesta de `fetch`, sus bytes o un modulo ya compilado. */
export type WasmSource = string | URL | Response | Promise<Response> | BufferSource | WebAssembly.Module;

/** Archivo de entrada: sus bytes o directamente el `File` de un `<input type="file">`. */
export type FileInput = Uint8Array | ArrayBuffer | Blob;

/**
 * Presupuesto de recursos de una conversion. Al alcanzar un limite, el documento se entrega leido
 * hasta ahi con un aviso en `warnings` (no se rechaza). Los que se omiten quedan por defecto.
 */
export interface Limits {
  /** Datos descomprimidos en total, en MB (todas las partes y flujos del archivo). Por defecto 128. */
  maxDecompressedMB?: number;
  /** Paginas de PDF que se leen. Por defecto 20000. */
  maxPages?: number;
  /** Caracteres de texto. Por defecto 50 000 000. */
  maxChars?: number;
}

export interface JsonOptions {
  /** Agrega estilo y alineacion de cada bloque, los tramos con formato y, en PDF, las lineas de cada pagina. */
  full?: boolean;
  /** Limites de recursos de esta conversion (por ejemplo, mas bajos en una funcion con poca memoria). */
  limits?: Limits;
}

/** Descarga y compila el modulo. Llamarla varias veces no lo descarga de nuevo. */
export declare function init(source: WasmSource): Promise<void>;

/** Convierte un .docx o un .pdf en JSON, como texto. Rechaza con `Error` si el archivo no se puede leer. */
export declare function docToJsonText(file: FileInput, options?: JsonOptions): Promise<string>;

/** Convierte un .docx o un .pdf en un objeto con metadatos, bloques de contenido y texto plano. */
export declare function docToJson(file: FileInput, options?: JsonOptions): Promise<DocDocument>;

/** Propiedades del archivo. Solo aparecen las que el documento trae; las fechas van en ISO 8601. */
export interface DocMetadata {
  title?: string;
  author?: string;
  subject?: string;
  keywords?: string;
  created?: string;
  modified?: string;
  application?: string;
  /** Solo PDF. */
  producer?: string;
  pdf_version?: string;
  encrypted?: boolean;
  [key: string]: string | number | boolean | undefined;
}

export interface DocStats {
  characters: number;
  words: number;
  headings: number;
  paragraphs: number;
  list_items: number;
  tables: number;
  images: number;
}

export interface DocCell {
  text: string;
  colspan?: number;
  rowspan?: number;
  /** Celda cubierta por una combinacion vertical iniciada mas arriba. */
  merged?: true;
}

export interface DocRun {
  text: string;
  bold?: true;
  italic?: true;
  underline?: true;
  strike?: true;
  link?: string;
}

export interface DocBlock {
  type: 'title' | 'heading' | 'paragraph' | 'list_item' | 'table' | 'image' | 'page_break';
  /** Titulos: 1 = principal. Items de lista: 0 = primer nivel. */
  level?: number;
  /** Items de lista: numerado o con vineta. */
  ordered?: boolean;
  /** Numeracion o vineta visible: "1.1", "a)", "•". */
  marker?: string;
  /** Notas y comentarios: su numero o su autor. */
  label?: string;
  /** Todos los bloques menos tablas, imagenes y saltos de pagina. */
  text?: string;
  /** Tablas: filas de celdas. */
  rows?: DocCell[][];
  /** Imagenes: nombre del archivo incrustado y texto alternativo. */
  name?: string;
  alt?: string;
  /** Solo PDF. */
  page?: number;
  /** Solo con `full`. */
  style?: string;
  align?: string;
  size?: number;
  bold?: true;
  runs?: DocRun[];
}

export interface DocLine {
  text: string;
  x: number;
  y: number;
  size: number;
  bold?: true;
}

export interface DocPage {
  number: number;
  width: number;
  height: number;
  text: string;
  images?: number;
  /** Solo con `full`. */
  lines?: DocLine[];
}

export interface DocOutlineItem {
  title: string;
  level: number;
  page?: number;
}

export interface DocDocument {
  type: 'docx' | 'pdf';
  metadata: DocMetadata;
  page_count: number | null;
  stats: DocStats;
  /** Bloques en orden de lectura. */
  content: DocBlock[];
  /** Solo PDF. */
  pages?: DocPage[];
  /** Solo PDF: marcadores (tabla de contenidos). */
  outline?: DocOutlineItem[];
  /** Solo Word, cuando existen. */
  headers?: DocBlock[];
  footers?: DocBlock[];
  footnotes?: DocBlock[];
  endnotes?: DocBlock[];
  comments?: DocBlock[];
  /** Avisos: paginas escaneadas sin texto, indice reconstruido, fuentes ilegibles. */
  warnings?: string[];
  /** Todo el texto plano. */
  text: string;
}
