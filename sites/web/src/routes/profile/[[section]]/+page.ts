import { error } from "@sveltejs/kit";
import {
  DEFAULT_PROFILE_SECTION,
  isProfileSection,
  type ProfileSectionId,
} from "$lib/components/layout/profile-sections";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ params }): { section: ProfileSectionId } => {
  if (params.section === undefined) return { section: DEFAULT_PROFILE_SECTION };
  if (!isProfileSection(params.section)) error(404, "Not found");
  return { section: params.section };
};
