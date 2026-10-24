// Final exams moved into a dialog on the schedules page; old /finals links (with their semester) open it there.
import { redirect } from "@sveltejs/kit";
import { finalsPath } from "$lib/core/routes";
import type { PageLoad } from "./$types";

export const load: PageLoad = ({ url }) => {
  redirect(307, finalsPath(url.searchParams.get("term")));
};
