<script lang="ts">
  import "./layout.css";
  import type { Snippet } from "svelte";
  import AppFooter from "$lib/components/shell/AppFooter.svelte";
  import AppHeader from "$lib/components/shell/AppHeader.svelte";
  import { setSession } from "$lib/components/shell/session";
  import { session } from "$lib/data";
  import { personal } from "$lib/state";
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
