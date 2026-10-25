<script lang="ts">
  import { ChevronDown, ChevronLeft, ChevronRight } from "@lucide/svelte";
  import { tick, type Snippet } from "svelte";
  import { MediaQuery } from "svelte/reactivity";
  import Avatar from "../ui/Avatar.svelte";
  import Button from "../ui/Button.svelte";
  import IconButton from "../ui/IconButton.svelte";
  import { readLocal, writeLocal } from "../ui/storage";
  import { provideProfileBand } from "./profile-band.svelte";
  import { RAIL_WIDTH } from "./rail-context";
  import { STABLE_GUTTER, stableGutter } from "./scrollbar-gutter";
  import { PROFILE_SECTIONS, profileHref, type ProfileSectionId } from "./profile-sections";

  let {
    active,
    name,
    detail,
    avatar,
    title,
    band = true,
    actions: actionsProp,
    filters: filtersProp,
    children,
    class: className = "",
  }: {
    /** The section being shown. */
    active: ProfileSectionId;
    /** The student's name and second line (department); omitted while unknown. */
    name?: string;
    detail?: string;
    /** Photo URL. */
    avatar?: string;
    /** Section heading; defaults to the section's label. */
    title?: string;
    /** Put the heading, actions and filters in a white band (Saved Courses); false keeps them on the page background (Friends). */
    band?: boolean;
    /** Buttons right of the heading (Figma Friends: "Share Link"); sections set theirs with profileBand(). */
    actions?: Snippet;
    /** Selects under the heading (Figma "Department", "Course Level"); sections set theirs with profileBand(). */
    filters?: Snippet;
    /** Section content. */
    children: Snippet;
    class?: string;
  } = $props();

  const KEY = "cmucourses:rail:profile";
  const desktop = new MediaQuery("(min-width: 64rem)");
  let collapsed = $state(readLocal(KEY) === "hidden");
  const current = $derived(PROFILE_SECTIONS.find((section) => section.id === active) ?? PROFILE_SECTIONS[0]);
  const heading = $derived(title ?? current.label);
  // The section inside can hand its own actions and filters to the band (profileBand()).
  const slot = provideProfileBand();
  const actions = $derived(actionsProp ?? slot.current.actions);
  const filters = $derived(filtersProp ?? slot.current.filters);

  let aside = $state<HTMLElement>();

  async function setCollapsed(next: boolean): Promise<void> {
    collapsed = next;
    writeLocal(KEY, next ? "hidden" : null);
    // The toggle button is replaced; keep keyboard focus on its counterpart.
    await tick();
    aside?.querySelector<HTMLElement>("[data-rail-toggle]")?.focus();
  }
</script>

{#snippet sectionHeader(showTitle: boolean)}
  <div class="flex flex-wrap items-center justify-between gap-3">
    <h1 class="text-heading-4 text-strong {showTitle ? '' : 'sr-only'}">{heading}</h1>
    {#if actions}<div class="flex flex-wrap items-center gap-3">{@render actions()}</div>{/if}
  </div>
  {#if filters}<div class="mt-4 flex flex-wrap gap-3">{@render filters()}</div>{/if}
{/snippet}

{#if desktop.current}
  <div class="flex min-h-0 flex-1 {className}">
    <aside
      bind:this={aside}
      aria-label="Profile"
      class="shrink-0 border-r border-line bg-surface {collapsed ? 'w-16' : RAIL_WIDTH}"
    >
      <div
        {@attach stableGutter}
        class="scrollbar-thin sticky top-(--header-h) max-h-[calc(100dvh-var(--header-h))] overflow-y-auto {STABLE_GUTTER}"
      >
        {#if collapsed}
          <div class="flex flex-col items-center gap-1 pt-5">
            <IconButton
              label="Show profile menu"
              icon={ChevronRight}
              aria-expanded="false"
              data-rail-toggle
              onclick={() => setCollapsed(false)}
            />
            <nav aria-label="Profile sections" class="mt-3">
              <ul class="flex flex-col items-center gap-1">
                {#each PROFILE_SECTIONS as section (section.id)}
                  <li>
                    <IconButton
                      href={profileHref(section.id)}
                      label={section.label}
                      icon={section.icon}
                      pressed={section.id === active}
                      aria-current={section.id === active ? "page" : undefined}
                    />
                  </li>
                {/each}
              </ul>
            </nav>
          </div>
        {:else}
          <!-- The same header as RailLayout's, so Hide and the title sit where they do on every rail. -->
          <div class="relative px-6 pt-[39px] min-[90rem]:px-[31px]">
            <h2 class="text-heading-5 text-ink">Profile</h2>
            <Button
              variant="ghost"
              trailingIcon={ChevronLeft}
              aria-label="Hide profile menu"
              aria-expanded="true"
              data-rail-toggle
              class="absolute top-[19px] right-2 min-[90rem]:right-[18px]"
              onclick={() => setCollapsed(true)}
            >
              Hide
            </Button>
          </div>
          <div class="flex items-center gap-4 px-6 pt-6 pb-[23px] min-[90rem]:px-[31px]">
            <Avatar {name} src={avatar} size="lg" decorative />
            <div class="min-w-0">
              <p class="truncate text-heading-5 text-strong">{name ?? "Your profile"}</p>
              {#if detail}<p class="truncate text-body-2 text-strong">{detail}</p>{/if}
            </div>
          </div>
          <!-- Full-width rows draw the focus ring inside, where the rail's scroll box can't cut it off. -->
          <nav aria-label="Profile sections" class="border-t border-line">
            <ul>
              {#each PROFILE_SECTIONS as section (section.id)}
                {@const isActive = section.id === active}
                <li>
                  <a
                    href={profileHref(section.id)}
                    aria-current={isActive ? "page" : undefined}
                    class="flex items-center gap-3 px-6 py-4 text-body-1 text-ink transition-colors focus-visible:-outline-offset-2 {isActive
                      ? 'bg-line'
                      : 'hover:bg-paper'}"
                  >
                    <section.icon aria-hidden="true" class="size-5 shrink-0" />
                    <span class="min-w-0 flex-1 truncate">{section.label}</span>
                    <ChevronRight aria-hidden="true" class="size-5 shrink-0" />
                  </a>
                </li>
              {/each}
            </ul>
          </nav>
        {/if}
      </div>
    </aside>

    <div class="flex min-w-0 flex-1 flex-col">
      {#if band}
        <div class="border-b border-line bg-surface p-8">{@render sectionHeader(true)}</div>
        <div class="flex-1 px-8 py-8">{@render children()}</div>
      {:else}
        <div class="flex-1 px-8 py-8">
          <div class="mb-6">{@render sectionHeader(true)}</div>
          {@render children()}
        </div>
      {/if}
    </div>
  </div>
{:else}
  <!-- Phones and tablets: the sections become an accordion; the open one shows its content. -->
  <div class="flex flex-1 flex-col bg-surface {className}">
    <div class="flex items-center gap-3 px-4 py-4">
      <Avatar {name} src={avatar} size="md" decorative />
      <div class="min-w-0">
        <p class="truncate text-heading-6 text-strong">{name ?? "Your profile"}</p>
        {#if detail}<p class="truncate text-body-2 text-muted">{detail}</p>{/if}
      </div>
    </div>
    <nav aria-label="Profile sections" class="border-t border-line">
      <ul>
        {#each PROFILE_SECTIONS as section (section.id)}
          {@const isActive = section.id === active}
          <li class="border-b border-line">
            <a
              href={profileHref(section.id)}
              aria-current={isActive ? "page" : undefined}
              class="flex items-center gap-3 px-6 py-[22px] text-body-1 text-ink focus-visible:-outline-offset-2 {isActive
                ? 'bg-paper'
                : 'hover:bg-paper'}"
            >
              <section.icon aria-hidden="true" class="size-5 shrink-0" />
              <span class="min-w-0 flex-1 truncate">{section.label}</span>
              <ChevronDown
                aria-hidden="true"
                class="size-5 shrink-0 transition-transform {isActive ? 'rotate-180' : ''}"
              />
            </a>
            {#if isActive}
              <div class="bg-paper px-4 pt-1 pb-6 sm:px-6">
                <div class="mb-4">{@render sectionHeader(false)}</div>
                {@render children()}
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    </nav>
  </div>
{/if}
