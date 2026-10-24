// The app shell. Import from "$lib/components/shell".
export { default as AppFooter } from "./AppFooter.svelte";
export { default as AppHeader } from "./AppHeader.svelte";
export { default as SignInPrompt } from "./SignInPrompt.svelte";
export { default as ThemeSwitch } from "./ThemeSwitch.svelte";
export { default as Title } from "./Title.svelte";
export { NAV_ITEMS, isActive } from "./nav";
export type { NavItem } from "./nav";
export { createStubSession, getSession, setSession } from "./session";
export type { SessionStatus, ShellSession } from "./session";
export { THEME_KEY, applyTheme, theme } from "./theme.svelte";
export type { ThemePreference } from "./theme.svelte";
