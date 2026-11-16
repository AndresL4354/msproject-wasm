/** Tipo MIME de un .xlsx, para crear el Blob de descarga. */
export declare const XLSX_MIME: string;

/** De donde sale mpp2xlsx.wasm: una URL, una respuesta de `fetch`, sus bytes o un modulo ya compilado. */
export type WasmSource = string | URL | Response | Promise<Response> | BufferSource | WebAssembly.Module;

/** Archivo de entrada: sus bytes o directamente el `File` de un `<input type="file">`. */
export type FileInput = Uint8Array | ArrayBuffer | Blob;

export interface JsonOptions {
  /** Agrega a cada tarea todos los campos estandar de Project, bajo `fields`. */
  full?: boolean;
}

/** Descarga y compila el modulo. Llamarla varias veces no lo descarga de nuevo. */
export declare function init(source: WasmSource): Promise<void>;

/** Convierte un .mpp en un libro de Excel (.xlsx). Rechaza con `Error` si el archivo no se puede leer. */
export declare function mppToXlsx(file: FileInput): Promise<Uint8Array>;

/** Convierte un .mpp en JSON, como texto. */
export declare function mppToJsonText(file: FileInput, options?: JsonOptions): Promise<string>;

/** Convierte un .mpp en un objeto con el proyecto, sus tareas, recursos y asignaciones. */
export declare function mppToJson(file: FileInput, options?: JsonOptions): Promise<MppProject>;

/** Unidades de duracion: "m", "h", "d", "w", "mo", "%" y sus variantes transcurridas "em", "eh", "ed", "ew", "emo", "e%". */
export type DurationUnits = string;

export interface Duration {
  value: number;
  units: DurationUnits;
}

/** Valor de un campo. Las fechas son texto "2026-05-20T08:00:00", en hora local sin zona. */
export type FieldValue = string | number | boolean | Duration;

export interface MppProjectInfo {
  title: string | null;
  author: string | null;
  last_author: string | null;
  company: string | null;
  manager: string | null;
  start: string | null;
  finish: string | null;
  status_date: string | null;
  minutes_per_day: number;
  minutes_per_week: number;
  days_per_month: number;
  default_duration_units: DurationUnits;
  default_calendar: string | null;
  currency_symbol: string | null;
  application: string;
  application_version: number;
  task_count: number;
  resource_count: number;
  assignment_count: number;
}

export interface MppPredecessorLink {
  /** Numero de fila de la predecesora (`null` si ya no existe). */
  id: number | null;
  uid: number;
  type: 'FS' | 'SS' | 'FF' | 'SF';
  lag: number;
  lag_units: DurationUnits | null;
}

export interface MppTaskResource {
  uid: number;
  id: number;
  name: string | null;
  /** 1 = una persona a tiempo completo. */
  units: number | null;
  work_hours: number | null;
}

export interface MppTask {
  /** Numero de fila en Project. */
  id: number;
  /** Identificador estable de la tarea dentro del archivo. */
  uid: number;
  name: string | null;
  outline_level: number;
  outline_number: string;
  wbs: string;
  parent_uid: number | null;
  parent_id: number | null;
  start: string | null;
  finish: string | null;
  duration: number | null;
  duration_units: DurationUnits | null;
  duration_hours: number | null;
  percent_complete: number | null;
  percent_work_complete: number | null;
  summary: boolean;
  milestone: boolean;
  critical: boolean;
  active: boolean;
  manual: boolean;
  work_hours: number | null;
  total_slack_hours: number | null;
  constraint_type: string | null;
  constraint_date: string | null;
  /** Como las muestra Project: "12;15FS+1d". */
  predecessors: string;
  successors: string;
  predecessor_links: MppPredecessorLink[];
  resource_names: string;
  resources: MppTaskResource[];
  notes: string | null;
  /** Campos personalizados con valor, por su alias si lo tienen ("Order Number") o por su nombre ("Text5"). */
  custom: Record<string, FieldValue>;
  /** Solo con `full`: todos los campos estandar, por nombre de campo de Project. */
  fields?: Record<string, FieldValue>;
}

export interface MppResource {
  id: number;
  uid: number;
  name: string | null;
  type: 'work' | 'material' | 'cost';
  group: string | null;
  initials: string | null;
  max_units: number | null;
  peak_units: number | null;
  work_hours: number | null;
  base_calendar: string | null;
  start: string | null;
  finish: string | null;
  notes: string | null;
  custom: Record<string, FieldValue>;
}

export interface MppAssignment {
  uid: number;
  task_uid: number;
  task_id: number | null;
  task_name: string | null;
  resource_uid: number | null;
  resource_id: number | null;
  resource_name: string | null;
  units: number | null;
  work_hours: number | null;
  remaining_work_hours: number | null;
  start: string | null;
  finish: string | null;
}

export interface MppCalendar {
  uid: number;
  name: string | null;
  base_calendar: string | null;
  resource_uid: number | null;
}

export interface MppProject {
  project: MppProjectInfo;
  /** Titulos de las columnas de la tabla de tareas, en el orden guardado en el archivo. */
  task_columns: string[];
  tasks: MppTask[];
  resources: MppResource[];
  assignments: MppAssignment[];
  calendars: MppCalendar[];
}
