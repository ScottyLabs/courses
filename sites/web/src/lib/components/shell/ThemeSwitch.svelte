<script lang="ts">
  import { Moon, Sun } from "@lucide/svelte";
  import Switch from "../ui/Switch.svelte";
  import { theme } from "./theme.svelte";

  let {
    showLabel = false,
    class: className = "",
  }: {
    /** Show "Dark theme" next to the switch (the phone menu). */
    showLabel?: boolean;
    class?: string;
  } = $props();
</script>

<Switch
  label="Dark theme"
  hideLabel={!showLabel}
  tone="neutral"
  icon={theme.dark ? Moon : Sun}
  checked={theme.dark}
  onchange={() => theme.toggle()}
  class={className}
/>
