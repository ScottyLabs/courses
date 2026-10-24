<script lang="ts">
  import { theme, type ThemePreference } from "$lib/components/shell/theme.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";

  const themes = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
    { value: "system", label: "System" },
  ];
</script>

<section aria-labelledby="appearance" class="max-w-xl">
  <h2 id="appearance" class="text-heading-6 text-strong">Appearance</h2>
  <p class="mt-1 mb-3 text-body-2 text-muted">Theme for this device.</p>
  <SegmentedControl
    options={themes}
    label="Theme"
    value={theme.preference}
    onchange={(value) => theme.set(value as ThemePreference)}
  />
</section>
