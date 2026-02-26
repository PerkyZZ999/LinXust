/// <reference types="vite/client" />

declare global {
  interface Window {
    linxustApi?: {
      helloFromRust: (name: string) => Promise<string>
    }
  }
}

export {}
