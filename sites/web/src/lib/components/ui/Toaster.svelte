<script lang="ts">
  import { CircleAlert, CircleCheck, Info, X } from "@lucide/svelte";
  import { showPanel, hidePanel } from "./floating";
  import { toasts, type ToastItem } from "./toast.svelte";

  let region = $state<HTMLElement>();
  const icons = { info: Info, success: CircleCheck, error: CircleAlert };
  const timers = new Map<number, ReturnType<typeof setTimeout>>();

  function schedule(item: ToastItem): void {
    if (item.duration <= 0 || timers.has(item.id)) return;
    timers.set(
      item.id,
      setTimeout(() => {
        timers.delete(item.id);
        toasts.dismiss(item.id);
      }, item.duration),
    );
  }

  function pause(item: ToastItem): void {
    clearTimeout(timers.get(item.id));
    timers.delete(item.id);
  }

  $effect(() => {
    for (const item of toasts.items) schedule(item);
  });

  // The region stays open (it is transparent and ignores the pointer) so its live
  // region exists before a toast arrives; each new toast raises it above any dialog.
  $effect(() => {
    if (region === undefined) return;
    const element = region;
    showPanel(element);
    return () => hidePanel(element);
  });

  $effect(() => {
    if (region !== undefined && toasts.items.length > 0) showPanel(region);
  });
</script>

<section
  bind:this={region}
  popover="manual"
  data-floating
  aria-label="Notifications"
  class="pointer-events-none fixed inset-x-0 bottom-0 flex w-full flex-col items-center gap-2 p-4 sm:items-end sm:p-6"
>
  <div aria-live="polite" class="flex w-full flex-col items-center gap-2 sm:items-end">
    {#each toasts.items as item (item.id)}
      {@const Icon = icons[item.kind]}
      <div
        role={item.kind === "error" ? "alert" : "status"}
        class="pointer-events-auto flex w-full max-w-sm items-start gap-3 rounded-card border border-line bg-surface px-4 py-3 text-body-2 text-ink shadow-dialog"
        onpointerenter={() => pause(item)}
        onpointerleave={() => schedule(item)}
        onfocusin={() => pause(item)}
        onfocusout={() => schedule(item)}
      >
        <Icon
          aria-hidden="true"
          class="mt-0.5 size-4 shrink-0 {item.kind === 'error'
            ? 'text-danger'
            : item.kind === 'success'
              ? 'text-ok'
              : 'text-accent-ink'}"
        />
        <p class="min-w-0 flex-1">{item.message}</p>
        {#if item.action}
          {@const action = item.action}
          <button
            type="button"
            class="shrink-0 rounded-full px-1.5 text-label-lg text-accent-ink hover:underline"
            onclick={() => {
              action.run();
              toasts.dismiss(item.id);
            }}
          >
            {action.label}
          </button>
        {/if}
        <button
          type="button"
          aria-label="Dismiss notification"
          class="-mr-1 grid size-6 shrink-0 place-items-center rounded-full text-muted hover:bg-hover hover:text-ink"
          onclick={() => toasts.dismiss(item.id)}
        >
          <X aria-hidden="true" class="size-4" />
        </button>
      </div>
    {/each}
  </div>
</section>
