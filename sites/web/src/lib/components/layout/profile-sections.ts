import { BookmarkPlus, Calendar1, CalendarPlus, Check, Settings, Users } from "@lucide/svelte";
import type { IconComponent } from "../ui/types";

export interface ProfileSection {
  id: string;
  label: string;
  icon: IconComponent;
}

/** Profile sections in navigation order; `/profile/[[section]]` takes these ids. */
export const PROFILE_SECTIONS = [
  { id: "saved", label: "Saved Courses", icon: BookmarkPlus },
  { id: "taking", label: "Currently Taking", icon: Calendar1 },
  { id: "taken", label: "Courses Taken", icon: Check },
  { id: "schedules", label: "Schedules", icon: CalendarPlus },
  { id: "friends", label: "Friends", icon: Users },
  { id: "settings", label: "Settings", icon: Settings },
] as const satisfies readonly ProfileSection[];

export type ProfileSectionId = (typeof PROFILE_SECTIONS)[number]["id"];

export const DEFAULT_PROFILE_SECTION: ProfileSectionId = "saved";

export function isProfileSection(value: string | undefined): value is ProfileSectionId {
  return PROFILE_SECTIONS.some((section) => section.id === value);
}

export function profileHref(id: ProfileSectionId): string {
  return `/profile/${id}`;
}
