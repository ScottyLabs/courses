<script lang="ts">
  import { BookOpen, CalendarPlus, ChevronRight, Ellipsis, UserMinus } from "@lucide/svelte";
  import Avatar from "$lib/components/ui/Avatar.svelte";
  import Card from "$lib/components/ui/Card.svelte";
  import CourseChip from "$lib/components/ui/CourseChip.svelte";
  import IconButton from "$lib/components/ui/IconButton.svelte";
  import Menu from "$lib/components/ui/Menu.svelte";
  import type { Friend } from "$lib/api";
  import { codeOf } from "$lib/core/requisite-format";
  import { coursePath } from "$lib/core/routes";

  // A friend on the Friends section (Figma "Profile Page (Friends)"): who they are, their shared schedule and the
  // courses they're taking.
  let {
    friend,
    courses,
    mine,
    termId,
    termName,
    title,
    onschedule,
    onremove,
  }: {
    friend: Friend;
    /** Course ids they're taking in the semester shown; null while loading. */
    courses: readonly string[] | null;
    /** Course ids the student is taking that semester, to point out the ones in common. */
    mine: ReadonlySet<string>;
    termId: string | null;
    termName: string;
    title: (courseId: string) => string | undefined;
    onschedule: () => void;
    onremove: () => void;
  } = $props();

  const headingId = $props.id();
  const sorted = $derived(
    (courses ?? []).toSorted(
      (a, b) => Number(mine.has(b)) - Number(mine.has(a)) || a.localeCompare(b),
    ),
  );
  const common = $derived(sorted.filter((id) => mine.has(id)).length);
  const sharesAnything = $derived(friend.sharesCourses || friend.sharesSchedules);
</script>

<Card as="article" padding="none" aria-labelledby={headingId} class="flex flex-col gap-4 p-5">
  <div class="flex items-start gap-4">
    <Avatar name={friend.name} size="md" tone="soft" decorative />
    <div class="min-w-0 flex-1">
      <h3 id={headingId} class="truncate text-heading-6 text-strong">{friend.name}</h3>
      <p class="truncate text-label-md font-normal text-ink">{friend.andrewId}</p>
    </div>
    <Menu
      label="Options for {friend.name}"
      items={[{ label: "Remove friend", icon: UserMinus, danger: true, onselect: onremove }]}
    >
      {#snippet trigger(props)}
        <IconButton {...props} label="Options for {friend.name}" icon={Ellipsis} size="md" class="-mt-2 -mr-2" />
      {/snippet}
    </Menu>
  </div>

  {#if friend.sharesSchedules}
    <button
      type="button"
      aria-label="{friend.name}'s schedules"
      aria-haspopup="dialog"
      onclick={onschedule}
      class="-mx-2 flex min-h-10 items-center gap-2 rounded-field px-2 text-left text-body-1 text-ink transition-colors hover:bg-hover"
    >
      <CalendarPlus aria-hidden="true" class="size-5 shrink-0" />
      <span class="flex-1">Schedule</span>
      <ChevronRight aria-hidden="true" class="size-5 shrink-0 text-muted" />
    </button>
  {:else}
    <p class="flex min-h-10 items-center gap-2 text-body-2 text-muted">
      <CalendarPlus aria-hidden="true" class="size-5 shrink-0" />Schedules not shared
    </p>
  {/if}

  <div class="flex flex-col gap-3">
    <p class="flex items-center gap-2 text-body-1 text-ink">
      <BookOpen aria-hidden="true" class="size-5 shrink-0" />
      <span>Courses in {termName}</span>
      {#if common > 0}
        <span class="ml-auto shrink-0 text-label-md font-normal text-ok">{common} in common</span>
      {/if}
    </p>
    {#if !sharesAnything}
      <p class="text-body-2 text-muted">{friend.name} doesn't share their courses.</p>
    {:else if courses === null}
      <p class="text-body-2 text-muted">Loading&hellip;</p>
    {:else if sorted.length === 0}
      <p class="text-body-2 text-muted">Nothing shared for {termName}.</p>
    {:else}
      <ul class="flex flex-wrap gap-2" aria-label="{friend.name}'s courses in {termName}">
        {#each sorted as id (id)}
          <li class="max-w-full" title={mine.has(id) ? "You're taking this too" : undefined}>
            <CourseChip
              code={codeOf(id)}
              title={title(id)}
              href={coursePath(id, termId)}
              tone={mine.has(id) ? "ok" : "neutral"}
            />
            {#if mine.has(id)}<span class="sr-only">(you're taking this too)</span>{/if}
          </li>
        {/each}
      </ul>
    {/if}
  </div>
</Card>
