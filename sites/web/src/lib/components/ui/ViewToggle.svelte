<script lang="ts" module>
  import { LayoutGrid, List } from "@lucide/svelte";
  import { readLocal, writeLocal } from "./storage";
  import type { IconComponent } from "./types";

  export type ListView = "cards" | "list";

  /** One of a page's own views, for `options`: an icon, with the label as its accessible name. */
  export interface ViewOption<V extends string = string> {
    value: V;
    label: string;
    icon: IconComponent;
  }

  const CARDS_AND_LIST: readonly ViewOption<ListView>[] = [
    { value: "cards", label: "Cards", icon: LayoutGrid },
    { value: "list", label: "List", icon: List },
  ];

  function key(pageKey: string): string {
    return `cmucourses:view:${pageKey}`;
  }

  /**
   * The view this device last chose for a page, or null. Read it when creating the page's
   * state so the first render already uses it: `let view = $state(storedView("courses") ?? "cards")`.
   */
  export function storedView(pageKey: string): ListView | null {
    return storedViewOf(pageKey, ["cards", "list"]);
  }

  /** The same for a page with its own views (`options`): the stored one of `values`, or null. */
  export function storedViewOf<V extends string>(pageKey: string, values: readonly V[]): V | null {
    const stored = readLocal(key(pageKey));
    return values.find((value) => value === stored) ?? null;
  }

  export function storeView(pageKey: string, view: string): void {
    writeLocal(key(pageKey), view);
  }

  /** The views to offer: the page's own, else cards and list (V is ListView whenever `options` is left out). */
  function viewsOf<V extends string>(options: readonly ViewOption<V>[] | undefined): readonly ViewOption<V>[] {
    return options ?? (CARDS_AND_LIST as readonly ViewOption<V>[]);
  }

  function initialView<V extends string>(pageKey: string, options: readonly ViewOption<V>[] | undefined): V {
    const views = viewsOf(options);
    return storedViewOf(pageKey, views.map((view) => view.value)) ?? views[0].value;
  }
</script>

<script lang="ts" generics="V extends string = ListView">
  import SegmentedControl from "./SegmentedControl.svelte";

  let {
    pageKey,
    options,
    value = $bindable(initialView(pageKey, options)),
    class: className = "",
  }: {
    /** Remembers the choice per page on this device, such as "courses" or "instructors". */
    pageKey: string;
    /** The page's own views, such as the profile's schedule previews and details. Defaults to Cards and List. */
    options?: readonly ViewOption<V>[];
    value?: V;
    class?: string;
  } = $props();

  const views = $derived(viewsOf(options));
  const segments = $derived(views.map((view) => ({ ...view, iconOnly: true })));

  function choose(next: string): void {
    const view = views.find((option) => option.value === next);
    if (view === undefined) return;
    value = view.value;
    storeView(pageKey, view.value);
  }
</script>

<SegmentedControl
  options={segments}
  label="View as"
  size="sm"
  class={className}
  bind:value={() => value, choose}
/>
