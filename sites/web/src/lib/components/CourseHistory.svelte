<script lang="ts">
  import { data } from "$lib/data";
  import type { Course, Term, User } from "$lib/contracts";
  let {
    status,
    terms,
    user,
    openCourse,
  }: {
    status: "taking" | "taken";
    terms: Term[];
    user: User | null;
    openCourse: (code: string) => void;
  } = $props();
  let entries = $state<(Course & { term_id?: string })[]>([]);
  let code = $state("");
  let term = $state("");
  let message = $state("");
  let busy = $state(false);
  let generation = 0;
  $effect(() => {
    const selected = status;
    const identity = user?.subject;
    const request = ++generation;
    entries = [];
    if (identity)
      void data
        .savedCourses(selected)
        .then((rows) => {
          if (request === generation) entries = rows;
        })
        .catch((error) => (message = error.message));
  });
  async function mutate(
    remove: (Course & { term_id?: string }) | undefined = undefined,
  ) {
    busy = true;
    try {
      if (remove)
        await data.removeHistory(remove.code, status, remove.term_id!);
      else {
        const digits = code.replace(/\D/g, "");
        if (digits.length !== 5 || !term)
          throw new Error(
            "Enter a five-digit course code and choose a semester.",
          );
        await data.setHistory(
          `${digits.slice(0, 2)}-${digits.slice(2)}`,
          status,
          term,
        );
        code = "";
      }
      entries = await data.savedCourses(status);
      message = "Course history saved.";
    } catch (error) {
      message =
        error instanceof Error
          ? error.message
          : "Unable to save course history.";
    } finally {
      busy = false;
    }
  }
</script>

{#if !user}<p>
    <a href="/auth/login">Sign in with CMU</a> to keep your course history.
  </p>
{:else}
  <p>
    Your history is private unless you explicitly enable sharing with friends.
  </p>
  <form
    onsubmit={(event) => {
      event.preventDefault();
      void mutate();
    }}
  >
    <input
      aria-label="Course code"
      placeholder="15-122"
      bind:value={code}
      required
    />
    <select aria-label="History semester" bind:value={term} required
      ><option value="">Choose semester</option>{#each terms as t}<option
          value={t.id}>{t.label}</option
        >{/each}</select
    >
    <button disabled={busy}>Add course</button>
  </form>
  {#each entries as course}
    <div class="history-row">
      <button onclick={() => openCourse(course.code)}
        >{course.code} {course.name}</button
      ><span>{course.term_id}</span><button
        disabled={busy}
        onclick={() => mutate(course)}>Remove</button
      >
    </div>
  {:else}<p>No courses recorded.</p>{/each}
{/if}
{#if message}<p role="status">{message}</p>{/if}

<style>
  .history-row,
  form {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    margin: 1rem 0;
    align-items: center;
  }
  input,
  select,
  button {
    padding: 0.5rem;
  }
</style>
