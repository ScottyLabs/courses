<script lang="ts">
  import ProfileLayout from "$lib/components/layout/ProfileLayout.svelte";
  import { PROFILE_SECTIONS } from "$lib/components/layout/profile-sections";
  import { getSession } from "$lib/components/shell/session";
  import SignInPrompt from "$lib/components/shell/SignInPrompt.svelte";
  import Title from "$lib/components/shell/Title.svelte";
  import FriendsSection, { friendsActions } from "$lib/features/profile/FriendsSection.svelte";
  import SavedSection from "$lib/features/profile/SavedSection.svelte";
  import SchedulesSection from "$lib/features/profile/SchedulesSection.svelte";
  import SettingsSection from "$lib/features/profile/SettingsSection.svelte";
  import TakenSection from "$lib/features/profile/TakenSection.svelte";
  import TakingSection from "$lib/features/profile/TakingSection.svelte";
  import type { PageProps } from "./$types";

  let { data }: PageProps = $props();
  const session = getSession();
  const section = $derived(PROFILE_SECTIONS.find((item) => item.id === data.section) ?? PROFILE_SECTIONS[0]);

  // Each section is its own component, owned by the feature it belongs to.
  const SECTIONS = {
    saved: SavedSection,
    taking: TakingSection,
    taken: TakenSection,
    schedules: SchedulesSection,
    friends: FriendsSection,
  };
</script>

<Title value={section.label} />

<ProfileLayout
  active={data.section}
  name={session.status === "signed-in" ? session.name : undefined}
  detail={session.status === "signed-in" ? session.detail : undefined}
  band={data.section !== "friends"}
  actions={data.section === "friends" && session.status === "signed-in" ? friendsActions : undefined}
>
  {#if data.section === "settings"}
    <SettingsSection />
  {:else if session.status !== "signed-in"}
    <SignInPrompt
      title="Sign in to see your profile"
      description="Saved courses, your record, schedules and friends are kept in your CMU account."
    />
  {:else}
    {@const Section = SECTIONS[data.section]}
    <Section title={section.label} />
  {/if}
</ProfileLayout>
