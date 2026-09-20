/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** Absolute API origin; leave empty to use the dev proxy or the serving origin. */
  readonly VITE_API_BASE_URL?: string
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}
