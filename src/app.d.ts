// See https://svelte.dev/docs/kit/types#app
declare global {
  /** App version from package.json, injected by Vite `define`. */
  const __APP_VERSION__: string;

  namespace App {
    // interface Error {}
    // interface Locals {}
    // interface PageData {}
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
