<script lang="ts">
  import "./layout.css";
  import type { Snippet } from "svelte";
  import { afterNavigate, goto } from "$app/navigation";
  import AppFooter from "$lib/components/shell/AppFooter.svelte";
  import AppHeader from "$lib/components/shell/AppHeader.svelte";
  import { setSession } from "$lib/components/shell/session";
  import { installUpdateReload, session } from "$lib/data";
  import { device, isAllSemesters, personal, trackOrigin } from "$lib/state";
  import { shellSession } from "$lib/state/shell-session";
  import { applyTheme, theme } from "$lib/components/shell/theme.svelte";
  import Toaster from "$lib/components/ui/Toaster.svelte";

  let { children }: { children: Snippet } = $props();

  // The shell reads the session through context: the data layer's session, with
  // sign-out saving the student's changes first.
  setSession(shellSession(session, personal));

  $effect(() => {
    applyTheme(theme.dark);
  });

  // A tab opened before a deploy reloads instead of failing to load the new build's pages.
  $effect(() => installUpdateReload());

  // "Back to plan" and the remembered semester both follow navigation.
  afterNavigate(({ to }) => {
    if (!to) return;
    trackOrigin(to.url.pathname, to.url.search);
    const term = to.url.searchParams.get("term");
    if (term && !isAllSemesters(term)) device.rememberTerm(term);
  });

  // After the sign-in round trip, go back to the page the student signed in from.
  $effect(() => {
    if (session.status !== "signed-in") return;
    const path = session.consumeReturnPath();
    if (path) void goto(path, { replaceState: true });
  });
</script>

<div class="flex min-h-dvh flex-col">
  <a
    href="#main"
    class="sr-only z-50 rounded-full bg-surface px-4 py-2 text-label-lg text-ink shadow-popover focus:not-sr-only focus:fixed focus:top-3 focus:left-3"
  >
    Skip to content
  </a>
  <AppHeader />
  <main id="main" tabindex="-1" class="flex min-w-0 flex-1 flex-col outline-none">
    {@render children()}
  </main>
  <AppFooter />
</div>
<Toaster />
