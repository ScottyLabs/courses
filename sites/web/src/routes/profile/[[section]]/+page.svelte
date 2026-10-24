<script lang="ts">
  import { BookmarkPlus, Calendar1, CalendarPlus, Check, Users } from "@lucide/svelte";
  import ProfileLayout from "$lib/components/layout/ProfileLayout.svelte";
  import { PROFILE_SECTIONS } from "$lib/components/layout/profile-sections";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import { theme, type ThemePreference } from "$lib/components/shell/theme.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import EmptyState from "$lib/components/ui/EmptyState.svelte";
  import SegmentedControl from "$lib/components/ui/SegmentedControl.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
  const session = getSession();
  const section = $derived(PROFILE_SECTIONS.find((item) => item.id === data.section) ?? PROFILE_SECTIONS[0]);

  const placeholders = {
    saved: {
      icon: BookmarkPlus,
      text: "Courses you save from the course list appear here.",
    },
    taking: {
      icon: Calendar1,
      text: "This semester's courses from your record appear here, with their sections and times.",
    },
    taken: {
      icon: Check,
      text: "Your academic record appears here: courses taken, transfer credit, exams and waivers, imported from SIO or Stellic.",
    },
    schedules: {
      icon: CalendarPlus,
      text: "Your schedules for every semester appear here.",
    },
    friends: {
      icon: Users,
      text: "Your friends, friend requests and invite links appear here, with what each friend shares.",
    },
  };
  const themes = [
    { value: "light", label: "Light" },
    { value: "dark", label: "Dark" },
    { value: "system", label: "System" },
  ];
</script>

<Title value={section.label} />

<ProfileLayout
  active={data.section}
  name={session.status === "signed-in" ? session.name : undefined}
  detail={session.status === "signed-in" ? session.detail : undefined}
  band={data.section !== "friends"}
>
  {#if data.section === "settings"}
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
  {:else if session.status !== "signed-in"}
    <SignInPrompt
      title="Sign in to see your profile"
      description="Saved courses, your record, schedules and friends are kept in your CMU account."
    />
  {:else}
    {@const placeholder = placeholders[data.section]}
    <EmptyState icon={placeholder.icon} title={section.label} description={placeholder.text} />
  {/if}
</ProfileLayout>
