<script lang="ts" module>
  import { readLocal, writeLocal } from "./storage";

  export type ListView = "cards" | "list";

  function key(pageKey: string): string {
    return `cmucourses:view:${pageKey}`;
  }

  /**
   * The view this device last chose for a page, or null. Read it when creating the page's
   * state so the first render already uses it: `let view = $state(storedView("courses") ?? "cards")`.
   */
  export function storedView(pageKey: string): ListView | null {
    const stored = readLocal(key(pageKey));
    return stored === "cards" || stored === "list" ? stored : null;
  }

  export function storeView(pageKey: string, view: ListView): void {
    writeLocal(key(pageKey), view);
  }
</script>

<script lang="ts">
  import { LayoutGrid, List } from "@lucide/svelte";
  import SegmentedControl from "./SegmentedControl.svelte";

  let {
    pageKey,
    value = $bindable(storedView(pageKey) ?? "cards"),
    class: className = "",
  }: {
    /** Remembers the choice per page on this device, such as "courses" or "instructors". */
    pageKey: string;
    value?: ListView;
    class?: string;
  } = $props();

  const options = [
    { value: "cards", label: "Cards", icon: LayoutGrid, iconOnly: true },
    { value: "list", label: "List", icon: List, iconOnly: true },
  ];
</script>

<SegmentedControl
  {options}
  label="View as"
  size="sm"
  class={className}
  bind:value={() => value, (next) => (value = next === "list" ? "list" : "cards")}
  onchange={(next) => storeView(pageKey, next === "list" ? "list" : "cards")}
/>
