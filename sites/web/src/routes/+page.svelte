<script lang="ts">
  import {
    ApiError,
    type Course,
    type Instructor,
    type Gened,
    type FinalExam,
    type Schedule,
    type Term,
    type User,
    type Friend,
  } from "$lib/contracts";
  import { data, clearLocalCatalog } from "$lib/data";
  import "./app.css";
  import CourseHistory from "$lib/components/CourseHistory.svelte";
  import Friends from "$lib/components/Friends.svelte";
  import logo from "$lib/assets/logo.png";
  import searchIcon from "$lib/assets/search.svg";
  import starIcon from "$lib/assets/star.svg";
  import clockIcon from "$lib/assets/clock.svg";
  import calendarIcon from "$lib/assets/calendar.svg";
  import graduationIcon from "$lib/assets/graduation.svg";
  import bookIcon from "$lib/assets/book.svg";
  import sunIcon from "$lib/assets/sun.svg";
  import logoutIcon from "$lib/assets/logout.svg";
  import userIcon from "$lib/assets/user.svg";
  import feedbackIcon from "$lib/assets/feedback.svg";
  import savedIcon from "$lib/assets/saved.svg";
  import genedIcon from "$lib/assets/gened.svg";
  import unitsIcon from "$lib/assets/units.svg";
  import profileSavedIcon from "$lib/assets/profile_saved.svg";
  import profileTakingIcon from "$lib/assets/profile_taking.svg";
  import profileTakenIcon from "$lib/assets/profile_taken.svg";
  import profileScheduleIcon from "$lib/assets/profile_schedule.svg";
  import profileFriendsIcon from "$lib/assets/profile_friends.svg";
  import checkIcon from "$lib/assets/check.svg";
  import externalIcon from "$lib/assets/external.svg";

  type View =
    | "courses"
    | "schedule"
    | "saved"
    | "instructors"
    | "geneds"
    | "finals"
    | "profile";
  const navigation: { id: View; label: string; icon: string }[] = [
    { id: "courses", label: "Courses", icon: "▦" },
    { id: "schedule", label: "Schedule", icon: "▤" },
    { id: "instructors", label: "Instructors", icon: "♙" },
    { id: "saved", label: "Saved Courses", icon: "☆" },
    { id: "geneds", label: "Gen eds", icon: "◈" },
    { id: "finals", label: "Finals", icon: "▣" },
    { id: "profile", label: "Profile", icon: "◉" },
  ];
  let view = $state<View>("courses");
  let user = $state<User | null>(null);
  let authLoading = $state(true);
  let authError = $state(false);
  let authMessage = $state("");
  let terms = $state<Term[]>([]);
  let term = $state("");
  let query = $state("");
  let department = $state("");
  let level = $state("");
  let units = $state("");
  let offeredIn = $state("");
  let minFce = $state("");
  let courseTab = $state("all");
  let courses = $state<Course[]>([]);
  let total = $state(0);
  let page = $state(1);
  let loading = $state(false);
  let error = $state("");
  let selectedCourse = $state<Course | null>(null);
  let detailError = $state("");
  let instructorQuery = $state("");
  let instructors = $state<Instructor[]>([]);
  let selectedInstructor = $state<Instructor | null>(null);
  let geneds = $state<Gened[]>([]);
  let genedQuery = $state("");
  let finals = $state<FinalExam[]>([]);
  let finalsQuery = $state("");
  let finalUpload = $state<HTMLInputElement>();
  let schedules = $state<Schedule[]>([]);
  let savedCourses = $state<Course[]>([]);
  let friends = $state<Friend[]>([]);
  let friendAdding = $state(false);
  let friendQuery = $state("");
  let friendError = $state("");
  let scheduleCourses = $state<Record<string, Course>>({});
  let draft = $state<Schedule>({
    id: "local",
    name: "Untitled",
    term_id: "",
    items: [],
  });
  let sharedView = $state(false);
  let message = $state("");
  let programs = $state<{ id: string; name: string; audit_name: string }[]>([]);
  let programSearch = $state("");
  let programDetails = $state<Awaited<ReturnType<typeof data.program>> | null>(
    null,
  );
  let programError = $state("");
  async function findPrograms() {
    try {
      programs = (await data.programs(programSearch)).items;
      programError = "";
    } catch (cause) {
      programError = reason(cause);
    }
  }
  async function chooseProgram(id: string) {
    saveDraft({ ...draft, program_id: id || undefined });
    if (!id) {
      programDetails = null;
      return;
    }
    try {
      programDetails = await data.program(id);
      programError = "";
    } catch (cause) {
      programError = reason(cause);
    }
  }
  function requirementText(tree: unknown): string {
    if (!tree || typeof tree !== "object") return "";
    const node = tree as {
      screen_name?: string;
      constraints?: { type_string?: string }[];
      choices?: unknown[];
    };
    return [
      node.screen_name,
      ...(node.constraints ?? []).map((c) => c.type_string),
      ...(node.choices ?? []).map(requirementText),
    ]
      .filter(Boolean)
      .join("\n");
  }

  let mobileMenu = $state(false);
  let filtersOpen = $state(false);
  let darkMode = $state(false);
  let courseSort = $state("relevance");
  let profileSection = $state<"saved" | "taking" | "taken" | "friends">(
    "saved",
  );
  let expandedCourses = $state<string[]>([]);
  let departmentOpen = $state(false);
  let offeredOpen = $state(false);
  let mobileSearchFocused = $state(false);
  let recentSearches = $state<string[]>([]);
  let timer: ReturnType<typeof setTimeout>;
  let requestId = 0;
  let termsRequestId = 0;
  const title = $derived(
    navigation.find((item) => item.id === view)?.label ?? "Courses",
  );
  const selectedCodes = $derived(draft.items.map((item) => item.course_code));
  const mobileSuggestions = $derived(
    courses
      .filter((course) =>
        `${course.code} ${course.name}`
          .toLowerCase()
          .includes(query.toLowerCase()),
      )
      .slice(0, 5),
  );
  const visibleGeneds = $derived(
    geneds.filter((item) =>
      `${item.name} ${item.description ?? ""}`
        .toLowerCase()
        .includes(genedQuery.toLowerCase()),
    ),
  );
  const visibleFinals = $derived(
    finals.filter((item) =>
      `${item.course_code} ${item.course_name}`
        .toLowerCase()
        .includes(finalsQuery.toLowerCase()),
    ),
  );
  const weekdays = ["M", "T", "W", "R", "F"];
  const events = $derived(
    draft.items.flatMap((item) => {
      const course = scheduleCourses[item.course_code];
      return (course?.sections ?? [])
        .filter((section) => item.section_ids.includes(section.id))
        .flatMap((section) =>
          section.meetings.flatMap((meeting) =>
            meeting.days.map((day) => ({
              day,
              code: item.course_code,
              label: section.label,
              start: meeting.starts_at,
              end: meeting.ends_at,
              location: meeting.location ?? "",
            })),
          ),
        );
    }),
  );
  function minutes(time: string) {
    const [hour, minute] = time.split(":").map(Number);
    return hour * 60 + minute;
  }
  function conflicts(event: (typeof events)[number]) {
    return events.some(
      (other) =>
        other !== event &&
        other.day === event.day &&
        minutes(other.start) < minutes(event.end) &&
        minutes(event.start) < minutes(other.end),
    );
  }

  function reason(value: unknown) {
    if (
      value instanceof ApiError &&
      [404, 501, 502, 503].includes(value.status)
    )
      return "This service is currently unavailable. Please try again later.";
    if (value instanceof ApiError && value.status === 401)
      return "Sign in with your CMU account to view this information.";
    return value instanceof Error
      ? value.message
      : "Unable to load data. Please try again.";
  }
  function saveDraft(next: Schedule) {
    if (sharedView) {
      next = { ...next, id: "local", share_id: undefined };
      sharedView = false;
      history.replaceState({}, "", `?view=${view}`);
    }
    draft = next;
    localStorage.setItem("cmucourses-draft", JSON.stringify(next));
  }
  async function toggleSaved(course: Course) {
    const exists = savedCourses.some((item) => item.code === course.code);
    try {
      if (user) {
        if (exists) await data.unsaveCourse(course.code);
        else await data.saveCourse(course.code);
      }
      savedCourses = exists
        ? savedCourses.filter((item) => item.code !== course.code)
        : [...savedCourses, course];
      if (!user)
        localStorage.setItem(
          "cmucourses-saved",
          JSON.stringify(
            savedCourses.map(
              ({
                code,
                name,
                description,
                units,
                department,
                offered_terms,
                gened_tags,
              }) => ({
                code,
                name,
                description,
                units,
                department,
                offered_terms,
                gened_tags,
              }),
            ),
          ),
        );
    } catch (cause) {
      message = reason(cause);
    }
  }
  async function refreshUser() {
    try {
      user = await data.me();
      authError = false;
      authMessage = "";
      try {
        savedCourses = await data.savedCourses();
      } catch (cause) {
        message = reason(cause);
        savedCourses = [];
      }
    } catch (cause) {
      user = null;
      authError = !(cause instanceof ApiError && cause.status === 401);
      authMessage = authError ? reason(cause) : "";
    } finally {
      authLoading = false;
    }
  }
  async function loadTerms() {
    const id = ++termsRequestId;
    try {
      const values = await data.terms();
      if (id === termsRequestId) {
        terms = values;
        if (!term) term = values[0]?.id ?? "";
      }
    } catch {
      /* Catalog API/local database may be unavailable. */
    }
  }
  async function signOut(event: MouseEvent) {
    event.preventDefault();
    await clearLocalCatalog().catch(() => {});
    location.href = "/auth/logout";
  }
  async function loadCourses() {
    const id = ++requestId;
    loading = true;
    error = "";
    try {
      const response = await data.courses({
        search: query,
        department,
        level,
        units,
        offered_in: offeredIn,
        min_fce: minFce,
        term,
        view: courseTab,
        page,
        page_size: 12,
      });
      if (id === requestId) {
        courses = response.items;
        total = response.total;
      }
    } catch (cause) {
      if (id === requestId) {
        courses = [];
        total = courses.length;
        error = reason(cause);
      }
    } finally {
      if (id === requestId) loading = false;
    }
  }
  function queueCourses() {
    page = 1;
    clearTimeout(timer);
    timer = setTimeout(loadCourses, 250);
  }
  function rememberSearch(value: string) {
    if (!value.trim()) return;
    recentSearches = [
      value.trim(),
      ...recentSearches.filter((item) => item !== value.trim()),
    ].slice(0, 5);
    localStorage.setItem(
      "cmucourses-recent-searches",
      JSON.stringify(recentSearches),
    );
    mobileSearchFocused = false;
    queueCourses();
  }
  async function openCourse(code: string) {
    selectedCourse = courses.find((course) => course.code === code) ?? {
      code,
      name: code,
      description: "",
      units: 0,
      department: "",
      offered_terms: [],
      gened_tags: [],
    };
    detailError = "";
    try {
      selectedCourse = await data.course(code, term);
    } catch (cause) {
      detailError = reason(cause);
    }
  }
  async function loadInstructors() {
    error = "";
    try {
      instructors = (await data.instructors(instructorQuery)).items;
    } catch (cause) {
      instructors = [];
      error = reason(cause);
    }
  }
  async function loadGeneds() {
    error = "";
    try {
      geneds = await data.geneds();
    } catch (cause) {
      geneds = [];
      error = reason(cause);
    }
  }
  async function loadFinals() {
    error = "";
    if (!term) {
      error = "Term data is unavailable.";
      return;
    }
    try {
      finals = await data.finals(term, selectedCodes);
    } catch (cause) {
      finals = [];
      error = reason(cause);
    }
  }
  async function loadFriends() {
    if (!user) return;
    try {
      friends = await data.friends();
      friendError = "";
    } catch (cause) {
      friendError = reason(cause);
    }
  }
  async function shareFriendInvite() {
    if (!user) {
      location.href = "/auth/login";
      return;
    }
    try {
      const { url } = await data.friendInvite();
      await navigator.clipboard.writeText(url);
      message = "Invite link copied.";
      friendError = "";
    } catch (cause) {
      friendError = reason(cause);
    }
  }
  async function addFriend() {
    if (!friendQuery.trim()) return;
    if (!user) {
      location.href = "/auth/login";
      return;
    }
    try {
      const friend = await data.addFriend(friendQuery.trim());
      friends = [...friends, friend];
      friendQuery = "";
      friendAdding = false;
      friendError = "";
    } catch (cause) {
      friendError = reason(cause);
    }
  }
  async function loadSchedules() {
    if (!user) return;
    try {
      schedules = await data.schedules();
    } catch (cause) {
      error = reason(cause);
    }
  }
  let scheduleRequestId = 0;
  async function loadScheduleDetails() {
    const request = ++scheduleRequestId;
    const selectedTerm = draft.term_id || term;
    const missing = draft.items
      .map((item) => item.course_code)
      .filter((code) => !scheduleCourses[code]);
    const found = await Promise.all(
      missing.map((code) => data.course(code, selectedTerm).catch(() => null)),
    );
    if (
      request !== scheduleRequestId ||
      selectedTerm !== (draft.term_id || term)
    )
      return;
    scheduleCourses = {
      ...scheduleCourses,
      ...Object.fromEntries(
        found
          .filter((course): course is Course => course !== null)
          .map((course) => [course.code, course]),
      ),
    };
  }
  async function openSchedule(selected: Schedule) {
    scheduleCourses = {};
    programDetails = null;
    programError = "";
    term = selected.term_id;
    saveDraft(selected);
    void loadScheduleDetails();
    if (selected.program_id) {
      try {
        const details = await data.program(selected.program_id);
        if (draft.program_id === selected.program_id) programDetails = details;
      } catch (cause) {
        programError = reason(cause);
      }
    }
  }
  function changeScheduleTerm() {
    scheduleCourses = {};
    saveDraft({
      ...draft,
      term_id: term,
      items: draft.items.map((item) => ({ ...item, section_ids: [] })),
    });
    void loadScheduleDetails();
    message =
      "Semester changed. Select sections again; remove courses not offered this term before saving.";
  }
  function go(next: View) {
    view = next;
    mobileMenu = false;
    error = "";
    const share =
      sharedView && next === "schedule" && draft.share_id
        ? `&share=${encodeURIComponent(draft.share_id)}`
        : "";
    history.replaceState({}, "", `?view=${next}${share}`);
    if (next === "courses") loadCourses();
    if (next === "instructors") loadInstructors();
    if (next === "geneds") loadGeneds();
    if (next === "finals") loadFinals();
    if (next === "schedule") {
      loadSchedules();
      loadScheduleDetails();
    }
  }
  function addCourse(course: Course) {
    if (draft.items.length && draft.term_id && term && draft.term_id !== term) {
      message =
        "Choose the schedule’s semester before adding this course, or start a new schedule.";
      return;
    }
    scheduleCourses = { ...scheduleCourses, [course.code]: course };
    if (!selectedCodes.includes(course.code))
      saveDraft({
        ...draft,
        term_id: term || draft.term_id,
        items: [...draft.items, { course_code: course.code, section_ids: [] }],
      });
    message = `${course.code} added to your schedule draft.`;
  }
  function toggleSection(code: string, id: string, checked: boolean) {
    if (!selectedCodes.includes(code) && selectedCourse)
      addCourse(selectedCourse);
    saveDraft({
      ...draft,
      items: draft.items.map((item) =>
        item.course_code === code
          ? {
              ...item,
              section_ids: checked
                ? [...item.section_ids, id]
                : item.section_ids.filter((section) => section !== id),
            }
          : item,
      ),
    });
  }
  async function copyShare() {
    if (sharedView && draft.share_id) {
      await navigator.clipboard.writeText(
        `${location.origin}${location.pathname}?view=schedule&share=${encodeURIComponent(draft.share_id)}`,
      );
      message = "Share link copied.";
      return;
    }
    if (!user) {
      if (authError) {
        message =
          "Account service unavailable. Try again when the API is running.";
        return;
      }
      location.href = "/auth/login";
      return;
    }
    if (!draft.term_id && !term) {
      message = "Choose a term before sharing.";
      return;
    }
    try {
      const base =
        draft.id === "local"
          ? await data.createSchedule(draft.name, draft.term_id || term)
          : draft;
      const saved = await data.saveSchedule({
        ...draft,
        id: base.id,
        revision: base.revision,
        term_id: draft.term_id || term,
      });
      const shareId =
        saved.share_id ?? (await data.shareSchedule(saved.id)).share_id;
      saveDraft({ ...saved, share_id: shareId });
      await navigator.clipboard.writeText(
        `${location.origin}${location.pathname}?view=schedule&share=${encodeURIComponent(shareId)}`,
      );
      message = "Share link copied.";
    } catch (cause) {
      message = reason(cause);
    }
  }
  async function saveSchedule() {
    if (!draft.term_id && !term) {
      message =
        "Choose a term before saving. Term data is currently unavailable.";
      return;
    }
    if (!user) {
      if (authError) {
        message =
          "Account service unavailable. Try again when the API is running.";
        return;
      }
      location.href = "/auth/login";
      return;
    }
    try {
      const base =
        draft.id === "local"
          ? await data.createSchedule(draft.name, draft.term_id || term)
          : draft;
      saveDraft(
        await data.saveSchedule({
          ...draft,
          id: base.id,
          revision: base.revision,
          term_id: draft.term_id || term,
        }),
      );
      await loadSchedules();
      message = "Schedule saved.";
    } catch (cause) {
      message = reason(cause);
    }
  }
  async function revokeScheduleShare() {
    try {
      await data.revokeScheduleShare(draft.id);
      saveDraft({ ...draft, share_id: undefined });
      message = "Share link disabled.";
    } catch (cause) {
      message = reason(cause);
    }
  }
  async function deleteSchedule(id: string) {
    try {
      await data.deleteSchedule(id);
      schedules = schedules.filter((item) => item.id !== id);
      if (draft.id === id)
        await openSchedule({
          id: "local",
          name: "Untitled",
          term_id: term,
          items: [],
        });
      message = "Schedule deleted.";
    } catch (cause) {
      message = reason(cause);
    }
  }
  async function uploadFinals(file: File | undefined) {
    if (!file) return;
    try {
      finals = await data.uploadFinals(file, term);
      message = "Final schedule updated.";
    } catch (cause) {
      error = reason(cause);
    }
  }
  function exportFinals() {
    const stamp = (value: string) =>
      new Date(value)
        .toISOString()
        .replace(/[-:]/g, "")
        .replace(/\.\d{3}/, "");
    const body = [
      "BEGIN:VCALENDAR",
      "VERSION:2.0",
      "PRODID:-//ScottyLabs//CMU Courses//EN",
      ...finals.flatMap((exam) => [
        "BEGIN:VEVENT",
        `UID:${exam.course_code}-${exam.starts_at}@cmu.courses`,
        `DTSTART:${stamp(exam.starts_at)}`,
        `DTEND:${stamp(exam.ends_at)}`,
        `SUMMARY:${exam.course_code} Final Exam`,
        `LOCATION:${(exam.location ?? "").replace(/,/g, "\\,")}`,
        "END:VEVENT",
      ]),
      "END:VCALENDAR",
    ].join("\r\n");
    const url = URL.createObjectURL(
      new Blob([body], { type: "text/calendar" }),
    );
    const link = document.createElement("a");
    link.href = url;
    link.download = "cmu-finals.ics";
    link.click();
    URL.revokeObjectURL(url);
  }
  $effect(() => {
    if (
      view === "profile" &&
      typeof window !== "undefined" &&
      window.innerWidth <= 800
    ) {
      requestAnimationFrame(() => {
        const nav = document.querySelector(".site-nav");
        if (nav) nav.scrollLeft = nav.scrollWidth;
      });
    }
  });
  let initialized = false;
  $effect(() =>
    data.subscribeCatalog((state) => {
      if (state.status !== "ready") return;
      void loadTerms().then(() => {
        if (view === "courses") loadCourses();
        if (view === "instructors") loadInstructors();
        if (view === "geneds") loadGeneds();
        if (view === "schedule") {
          scheduleCourses = {};
          loadScheduleDetails();
        }
      });
    }),
  );
  $effect(() => {
    if (initialized) return;
    initialized = true;
    void (async () => {
      const incomingInvite = new URLSearchParams(location.search).get("invite");
      if (incomingInvite) sessionStorage.setItem("cmucourses-friend-invite", incomingInvite);
      const requested = new URLSearchParams(location.search).get("view");
      if (navigation.some((item) => item.id === requested))
        view = requested as View;
      if (
        new URLSearchParams(location.search).has("invite") ||
        sessionStorage.getItem("cmucourses-friend-invite")
      ) {
        view = "profile";
        profileSection = "friends";
      }
      try {
        const stored = localStorage.getItem("cmucourses-draft");
        if (stored) {
          const parsed = JSON.parse(stored);
          if (Array.isArray(parsed.items)) draft = parsed;
        }
      } catch {
        /* damaged draft */
      }
      try {
        const stored = localStorage.getItem("cmucourses-saved");
        if (stored) {
          const parsed = JSON.parse(stored);
          if (Array.isArray(parsed))
            savedCourses = parsed.map(
              ({
                code,
                name,
                description,
                units,
                department,
                offered_terms,
                gened_tags,
              }) => ({
                code,
                name,
                description,
                units,
                department,
                offered_terms,
                gened_tags,
              }),
            );
        }
      } catch {
        /* damaged saved list */
      }
      try {
        const stored = localStorage.getItem("cmucourses-recent-searches");
        if (stored) {
          const parsed = JSON.parse(stored);
          if (Array.isArray(parsed)) recentSearches = parsed;
        }
      } catch {
        /* damaged recent searches */
      }
      await Promise.all([refreshUser(), loadTerms()]);
      if (view === "courses") await loadCourses();
      const share = new URLSearchParams(location.search).get("share");
      if (share) {
        try {
          draft = { ...(await data.sharedSchedule(share)), share_id: share };
          sharedView = true;
          view = "schedule";
        } catch (cause) {
          message = reason(cause);
        }
      }
      if (view !== "courses") go(view);
    })();
  });
</script>

<svelte:head
  ><title>{title} · CMU Courses</title><meta
    name="description"
    content="Explore CMU courses, instructors, schedules, gen eds, and finals."
  /></svelte:head
>
<svelte:window
  onkeydown={(event) => {
    if (event.key === "Escape") {
      selectedCourse = null;
      selectedInstructor = null;
      mobileMenu = false;
    }
  }}
/>
<div class:dark={darkMode} class="app-shell">
  <header class="site-header">
    <button
      class="mobile-menu"
      aria-label="Open navigation"
      onclick={() => (mobileMenu = !mobileMenu)}>☰</button
    >
    <button class="site-brand" onclick={() => go("courses")}
      ><img src={logo} alt="" /><span>CMU Courses</span></button
    >
    <nav class:open={mobileMenu} class="site-nav" aria-label="Main navigation">
      <button class:active={view === "courses"} onclick={() => go("courses")}
        ><img src={searchIcon} alt="" />Courses</button
      >
      <button
        class:active={view === "instructors"}
        onclick={() => go("instructors")}
        ><img src={userIcon} alt="" />Instructors</button
      >
      {#if view === "courses" || view === "instructors" || view === "saved"}<button
          class:active={view === "saved"}
          onclick={() => go("saved")}
          ><img src={savedIcon} alt="" />Saved</button
        >{/if}
      <button class:active={view === "schedule"} onclick={() => go("schedule")}
        ><img src={clockIcon} alt="" />Schedules</button
      >
      {#if view === "finals"}<button
          class:active={view === "finals"}
          onclick={() => go("finals")}
          ><img src={calendarIcon} alt="" />Final Schedule</button
        >{/if}
      {#if view === "profile" || view === "saved" || view === "schedule" || view === "finals"}<button
          class:active={view === "profile"}
          onclick={() => go("profile")}
          ><img src={userIcon} alt="" />Profile</button
        >{/if}
    </nav>
    <div class="header-actions">
      <button
        class="theme-button"
        aria-label="Toggle theme"
        onclick={() => (darkMode = !darkMode)}
        ><img src={sunIcon} alt="" /></button
      >{#if user}<button
          class="mobile-avatar"
          aria-label="Profile"
          onclick={() => go("profile")}>{user.name[0]}</button
        ><a class="signout" href="/auth/logout" onclick={signOut}
          ><img src={logoutIcon} alt="" />Sign out</a
        >{:else if authLoading}<button
          class="mobile-avatar"
          aria-label="Profile"
          onclick={() => go("profile")}>?</button
        ><span class="auth-status">Checking session…</span
        >{:else if authError}<button
          class="mobile-avatar"
          aria-label="Profile"
          onclick={() => go("profile")}>?</button
        ><button class="signout" onclick={refreshUser}>Retry account</button
        >{:else}<button
          class="mobile-avatar"
          aria-label="Profile"
          onclick={() => go("profile")}>?</button
        ><a class="signout" href="/auth/login">Sign in</a>{/if}
    </div>
  </header>
  {#if authMessage}<div role="alert" class="auth-notice">{authMessage} <a href="/auth/logout">Sign out</a></div>{/if}
  <div class="app-body">
    {#if view === "courses"}
      <aside class:visible={filtersOpen} class="filter-rail">
        <div class="mobile-filter-header">
          <div>
            <button
              class="theme-button"
              aria-label="Toggle theme"
              onclick={() => (darkMode = !darkMode)}
              ><img src={sunIcon} alt="" /></button
            ><span class="filter-avatar">{user?.name?.[0] ?? "CT"}</span>
          </div>
          <div>
            <button onclick={() => (filtersOpen = false)}
              >‹ &nbsp; Search</button
            >
            <h1>Filter</h1>
          </div>
        </div>
        <div class="rail-heading">
          <h2>Filter by</h2>
          <button onclick={() => (filtersOpen = !filtersOpen)}
            >Hide <span>‹</span></button
          >
        </div>
        <div
          class:expanded={departmentOpen}
          data-filter="department"
          class="rail-section"
        >
          <button
            type="button"
            class="filter-group-label"
            onclick={() => (departmentOpen = !departmentOpen)}
            ><img src={graduationIcon} alt="" />Department
            <span>⌃</span></button
          ><input
            id="department"
            placeholder="Search departments"
            bind:value={department}
            oninput={queueCourses}
          />
        </div>
        <div class="rail-section">
          <label for="units"
            ><img src={unitsIcon} alt="" />Units <span>⌃</span></label
          >
          <div
            class="unit-range"
            style={`--unit-progress:${((Number(units) || 12) / 24) * 100}%`}
          >
            <input
              id="units"
              aria-label="Minimum units"
              type="range"
              min="0"
              max="24"
              step="3"
              value={units || "12"}
              oninput={(event) => {
                units = event.currentTarget.value;
                queueCourses();
              }}
            /><small>0–24</small>
          </div>
          <div class="unit-chips">
            {#each ["6", "9", "12", "15"] as value}<button
                class:active={units === value}
                onclick={() => {
                  units = units === value ? "" : value;
                  queueCourses();
                }}>{value} Units</button
              >{/each}
          </div>
        </div>
        <div
          class:expanded={offeredOpen}
          data-filter="offered"
          class="rail-section compact"
        >
          <button
            type="button"
            class="filter-group-label"
            onclick={() => (offeredOpen = !offeredOpen)}
            ><img src={calendarIcon} alt="" />Offered in <span>⌃</span></button
          ><select id="offered" bind:value={offeredIn} onchange={queueCourses}
            ><option value="">Any semester</option><option value="fall"
              >Fall</option
            ><option value="spring">Spring</option><option value="summer"
              >Summer</option
            ></select
          >
        </div>
        <div class="rail-section compact">
          <label for="level"
            ><img src={bookIcon} alt="" />Course Level <span>⌄</span></label
          ><select id="level" bind:value={level} onchange={queueCourses}
            ><option value="">None</option><option value="100">100 level</option
            ><option value="200">200 level</option><option value="300"
              >300 level</option
            ><option value="400">400 level</option><option value="500"
              >500+</option
            ></select
          >
        </div>
        <div class="rail-section compact">
          <label for="gened-filter"
            ><img src={genedIcon} alt="" />GenEd <span>⌄</span></label
          ><select
            id="gened-filter"
            onchange={(event) => {
              if (event.currentTarget.value === "all") go("geneds");
              else {
                query = event.currentTarget.value;
                queueCourses();
              }
            }}
            ><option value="">Select College</option><option value="all"
              >Browse all GenEds</option
            ><option value="Dietrich">Dietrich</option><option value="SCS"
              >SCS</option
            ><option value="CIT">CIT</option></select
          ><select aria-label="GenEd category" disabled
            ><option>Select GenEd Category</option></select
          >
        </div>
        <button
          class="reset-button"
          onclick={() => {
            department = "";
            level = "";
            units = "";
            offeredIn = "";
            minFce = "";
            queueCourses();
          }}>Reset</button
        >
        <div class="aggregate">
          <h2>Aggregate FCEs</h2>
          <label
            >Semesters to show <select
              aria-label="FCE semesters"
              bind:value={minFce}
              onchange={queueCourses}
              ><option value="">None</option><option value="3">3.0+</option
              ><option value="4">4.0+</option><option value="4.5">4.5+</option
              ></select
            ></label
          >
          <div class="checks">
            <label><input type="checkbox" checked />Spring</label><label
              ><input type="checkbox" />Summer</label
            ><label><input type="checkbox" checked />Fall</label>
          </div>
        </div>
      </aside>
      <div class="work-area">
        <div class="catalog-search">
          <div class="mobile-search-title"><h1>Search</h1></div>
          <div class="search-line">
            <img src={searchIcon} alt="" /><input
              class="desktop-search-input"
              aria-label="Search courses"
              placeholder="Search courses by ID, description, name or keyword..."
              bind:value={query}
              oninput={queueCourses}
            /><input
              class="mobile-search-input"
              aria-label="Search courses"
              placeholder="Search courses"
              bind:value={query}
              oninput={queueCourses}
              onfocus={() => (mobileSearchFocused = true)}
              onkeydown={(event) => {
                if (event.key === "Enter") rememberSearch(query);
                if (event.key === "Escape") mobileSearchFocused = false;
              }}
            />
          </div>
          {#if mobileSearchFocused && query.trim()}<div
              class="mobile-suggestions"
            >
              {#each mobileSuggestions as suggestion}<button
                  onpointerdown={(event) => {
                    event.preventDefault();
                    query = suggestion.code;
                    rememberSearch(query);
                  }}>{suggestion.code}: {suggestion.name}</button
                >{/each}{#if recentSearches.length}<div
                  class="suggestions-label"
                >
                  ◷ &nbsp; Recent Searches
                </div>
                {#each recentSearches as recent}<button
                    onpointerdown={(event) => {
                      event.preventDefault();
                      query = recent;
                      rememberSearch(recent);
                    }}>× &nbsp; {recent}</button
                  >{/each}{/if}
            </div>{/if}
          <div class="mobile-filter-row">
            <button onclick={() => (filtersOpen = true)}
              >☰ &nbsp; Filter {Number(Boolean(department)) +
                Number(Boolean(units)) +
                Number(Boolean(level)) +
                Number(Boolean(offeredIn)) || ""}</button
            ><span
              >{total.toLocaleString()}
              {total === 1 ? "result" : "results"}</span
            >
          </div>
          {#if department || units || level || offeredIn}<div
              class="mobile-active-filters"
            >
              {#if department}<button
                  onclick={() => {
                    department = "";
                    queueCourses();
                  }}>{department} ×</button
                >{/if}{#if units}<button
                  onclick={() => {
                    units = "";
                    queueCourses();
                  }}>{units} Units ×</button
                >{/if}{#if level}<button
                  onclick={() => {
                    level = "";
                    queueCourses();
                  }}>{level} level ×</button
                >{/if}{#if offeredIn}<button
                  onclick={() => {
                    offeredIn = "";
                    queueCourses();
                  }}>{offeredIn} ×</button
                >{/if}<button
                class="mobile-filter-reset"
                onclick={() => {
                  department = "";
                  units = "";
                  level = "";
                  offeredIn = "";
                  queueCourses();
                }}>Reset</button
              >
            </div>{/if}
          <div class="search-options">
            <div>
              <span
                >{loading
                  ? "Searching…"
                  : `${total.toLocaleString()} ${total === 1 ? "result" : "results"}`}</span
              >
              <div class="active-filters">
                {#if department}<button
                    onclick={() => {
                      department = "";
                      queueCourses();
                    }}>{department} ×</button
                  >{/if}{#if units}<button
                    onclick={() => {
                      units = "";
                      queueCourses();
                    }}>{units} Units ×</button
                  >{/if}
              </div>
            </div>
            <select
              aria-label="Show courses"
              bind:value={courseSort}
              onchange={() => {
                courseTab = courseSort === "fces" ? "fces" : "all";
                queueCourses();
              }}
              ><option value="relevance">Show</option><option value="fces"
                >FCEs</option
              ><option value="all">All courses</option></select
            >
          </div>
        </div>
        <div class="course-results">
          {#if error}<p class="preview-notice">
              {error} <button onclick={loadCourses}>Retry</button>
            </p>{/if}{#if !loading && !courses.length}<div class="status-panel">
              <h3>No courses found</h3>
              <p>Try another search or reset your filters.</p>
            </div>{:else}{#each courses as course}<article
                class="figma-course-card"
              >
                <div class="course-main">
                  <button
                    class="course-title"
                    onclick={() => openCourse(course.code)}
                    ><strong>{course.code}</strong> {course.name}</button
                  >
                  <p class="course-department">{course.department}</p>
                  <div class="course-tags">
                    {#each course.gened_tags?.slice(0, 3) ?? [] as tag}<span
                        ><img src={externalIcon} alt="" />{tag}</span
                      >{/each}
                  </div>
                  <div class="requirements">
                    <span
                      ><b>Prerequisites:</b>
                      {#if course.prerequisites
                        ?.toLowerCase()
                        .includes("satisfied")}<img
                          src={checkIcon}
                          alt=""
                        />{/if}{course.prerequisites || "None"}</span
                    ><span
                      ><b>Corequisites:</b>
                      {#if course.corequisites?.some((value) => value
                          .toLowerCase()
                          .includes("satisfied"))}<img
                          src={checkIcon}
                          alt=""
                        />{/if}{course.corequisites?.join(", ") || "None"}</span
                    >
                  </div>
                </div>
                <div class="course-data">
                  <span>{course.units} units</span><span class="icon-value"
                    ><img
                      src={clockIcon}
                      alt=""
                    />{course.fce?.hours_per_week?.toFixed(1) ?? "—"} hrs/week</span
                  ><label class="icon-value"
                    ><img src={calendarIcon} alt="" /><select
                      aria-label={`Term for ${course.code}`}
                      onchange={(event) => {
                        term = event.currentTarget.value;
                        queueCourses();
                      }}
                      ><option value="">Select term</option
                      >{#each terms as option}<option
                          value={option.id}
                          selected={term === option.id}>{option.label}</option
                        >{/each}</select
                    ></label
                  >
                </div>
                <div class="course-card-actions">
                  <button
                    aria-label={`${savedCourses.some((item) => item.code === course.code) ? "Unsave" : "Save"} ${course.code}`}
                    onclick={() => toggleSaved(course)}
                    ><img src={starIcon} alt="" /></button
                  >
                  <div class="course-avatars">
                    <span>{course.instructors?.[0]?.[0] ?? "C"}</span><span
                      >{course.instructors?.[1]?.[0] ?? "S"}</span
                    ><small
                      >+{Math.max(
                        0,
                        (course.instructors?.length ?? 0) - 2,
                      )}</small
                    >
                  </div>
                </div>
                <p
                  class:expanded={expandedCourses.includes(course.code)}
                  class="course-description"
                >
                  {course.description}
                </p>
                <button
                  class="expand-course"
                  onclick={() =>
                    (expandedCourses = expandedCourses.includes(course.code)
                      ? expandedCourses.filter((code) => code !== course.code)
                      : [...expandedCourses, course.code])}
                  >{expandedCourses.includes(course.code)
                    ? "⌃ Collapse"
                    : "⌄ Expand All"}</button
                >
              </article>{/each}{#if total > 12}<div class="pager">
                <button
                  disabled={page === 1}
                  onclick={() => {
                    page--;
                    loadCourses();
                  }}>Previous</button
                ><span>Page {page} of {Math.ceil(total / 12)}</span><button
                  disabled={page >= Math.ceil(total / 12)}
                  onclick={() => {
                    page++;
                    loadCourses();
                  }}>Next</button
                >
              </div>{/if}{/if}
        </div>
      </div>
    {:else if view === "schedule"}
      <aside class="filter-rail schedule-rail">
        <div class="rail-heading">
          <h2>Schedule</h2>
          <button
            onclick={() =>
              openSchedule({
                id: "local",
                name: "Untitled",
                term_id: term,
                items: [],
              })}>⊕</button
          >
        </div>
        <div class="schedule-rail-section">
          <label for="schedule-select">Schedule</label><select
            id="schedule-select"
            onchange={(event) => {
              const selected = schedules.find(
                (item) => item.id === event.currentTarget.value,
              );
              if (selected) void openSchedule(selected);
            }}
            ><option value="">Select Schedule</option
            >{#each schedules as saved}<option value={saved.id}
                >{saved.name}</option
              >{/each}</select
          >
        </div>
        <div class="schedule-rail-section">
          <h2>Schedule Calendar</h2>
          <label for="schedule-term">Semester</label><select
            id="schedule-term"
            bind:value={term}
            onchange={changeScheduleTerm}
            ><option value="">Select Semester</option
            >{#each terms as option}<option value={option.id}
                >{option.label}</option
              >{/each}</select
          >{#each draft.items as item}<div class="schedule-choice">
              <label
                ><input type="checkbox" checked />{item.course_code} (Select Section)</label
              >
              <div>
                {#each ["A", "B", "C", "D", "E"] as letter}<button
                    onclick={() => openCourse(item.course_code)}
                    >{letter}</button
                  >{/each}
              </div>
            </div>{/each}
        </div>
        <div class="aggregate">
          <h2>Aggregate FCEs</h2>
          <label>Semesters to show <select><option>None</option></select></label
          >
          <div class="checks">
            <label><input type="checkbox" checked />Spring</label><label
              ><input type="checkbox" />Summer</label
            ><label><input type="checkbox" checked />Fall</label>
          </div>
        </div>
      </aside>
      <div class="work-area">
        <div class="schedule-top">
          <div class="schedule-title">
            <input
              aria-label="Schedule name"
              bind:value={draft.name}
              onchange={() => saveDraft(draft)}
            /><span>✎</span>
          </div>
          <div class="search-line">
            <img src={searchIcon} alt="" /><input
              aria-label="Search courses to add"
              placeholder="Search courses by ID, description, name or keyword..."
              bind:value={query}
              oninput={queueCourses}
              onkeydown={(event) => {
                if (event.key === "Enter") go("courses");
              }}
            />
          </div>
          <div class="schedule-chips">
            {#each draft.items as item}<button
                onclick={() =>
                  saveDraft({
                    ...draft,
                    items: draft.items.filter(
                      (entry) => entry.course_code !== item.course_code,
                    ),
                  })}>{item.course_code} ×</button
              >{/each}<select
              aria-label="Schedule actions"
              onchange={(event) => {
                if (event.currentTarget.value === "save") saveSchedule();
                if (event.currentTarget.value === "share") copyShare();
                if (event.currentTarget.value === "finals") go("finals");
                if (event.currentTarget.value === "delete")
                  deleteSchedule(draft.id);
                if (event.currentTarget.value === "revoke")
                  revokeScheduleShare();
                event.currentTarget.value = "";
              }}
              ><option value="">Show</option><option value="save"
                >Save schedule</option
              ><option value="share">Copy share link</option><option
                value="finals">Final Schedule</option
              >{#if user && draft.id !== "local" && !sharedView}<option
                  value="delete">Delete schedule</option
                >{#if draft.share_id}<option value="revoke"
                    >Disable share link</option
                  >{/if}{/if}</select
            >
          </div>
          <p class="workload-label">Total Workload</p>
          <p class="workload-value">
            {draft.items.reduce(
              (sum, item) =>
                sum + (scheduleCourses[item.course_code]?.units ?? 0),
              0,
            )} Units <span>|</span>
            {draft.items
              .reduce(
                (sum, item) =>
                  sum +
                  (scheduleCourses[item.course_code]?.fce?.hours_per_week ?? 0),
                0,
              )
              .toFixed(2)} hrs/week
          </p>
          {#if message}<p class="notice">{message}</p>{/if}
        </div>
        <details
          class="program-requirements"
          ontoggle={(event) => {
            if (event.currentTarget.open && draft.program_id)
              void data
                .program(draft.program_id)
                .then((value) => (programDetails = value))
                .catch((cause) => (programError = reason(cause)));
          }}
        >
          <summary>Major requirements</summary>
          <p>
            Choose a catalog or entry-year requirement set to reference while
            planning.
          </p>
          <form
            onsubmit={(event) => {
              event.preventDefault();
              findPrograms();
            }}
          >
            <input
              aria-label="Search major requirements"
              bind:value={programSearch}
              placeholder="Search majors, minors, or entry year"
            /><button type="submit">Find programs</button>
          </form>
          <select
            aria-label="Program requirements"
            value={draft.program_id ?? ""}
            onchange={(event) => chooseProgram(event.currentTarget.value)}
            ><option value="">Choose a program</option
            >{#each programs as program}<option value={program.id}
                >{program.name} {program.audit_name ?? ""}</option
              >{/each}</select
          >{#if programError}<p>{programError}</p>{/if}{#if programDetails}<h3>
              {programDetails.name}
            </h3>
            {#if programDetails.source_url}<a
                href={programDetails.source_url}
                target="_blank"
                rel="noreferrer"
                >Official catalog {programDetails.catalog_year ?? ""}</a
              >{/if}
            <p>
              Reference requirements; completion and substitutions must be
              confirmed with your advisor.
            </p>
            <pre
              style="white-space: pre-wrap; max-height: 24rem; overflow: auto">{programDetails.requirements_text ??
                requirementText(programDetails.tree)}</pre>{/if}
        </details>
        <div class="weekly-calendar">
          <div class="hour-spacer"></div>
          {#each weekdays as day}<div class="calendar-heading">
              {(
                { M: "Mon", T: "Tue", W: "Wed", R: "Thu", F: "Fri" } as Record<
                  string,
                  string
                >
              )[day]}
            </div>{/each}{#each Array.from({ length: 13 }, (_, i) => i + 8) as hour}<div
              class="hour-label"
            >
              {hour > 12 ? hour - 12 : hour}
              {hour < 12 ? "AM" : "PM"}
            </div>
            {#each weekdays as day}<div class="calendar-cell">
                {#each events.filter((event) => event.day === day && Math.floor(minutes(event.start) / 60) === hour) as event}<div
                    class:conflict={conflicts(event)}
                    class="calendar-event"
                  >
                    <b>{event.code}</b><span>{event.start}–{event.end}</span>
                  </div>{/each}
              </div>{/each}{/each}
        </div>
      </div>
    {:else if view === "finals"}
      <aside class="filter-rail">
        <div class="rail-heading"><h2>Filter by</h2></div>
        <div class="rail-section">
          <label for="final-dept"
            ><img src={graduationIcon} alt="" />Department</label
          ><input
            id="final-dept"
            placeholder="Department"
            bind:value={department}
          />
        </div>
      </aside>
      <div class="work-area finals-area">
        <div class="finals-header">
          <h1>Final Schedule</h1>
          <p>
            1. Download Calendar Export From SIO<br />2. Import the .ics file
            below
          </p>
          <div>
            <input
              type="file"
              accept=".ics,text/calendar"
              bind:this={finalUpload}
              hidden
              onchange={(event) => uploadFinals(event.currentTarget.files?.[0])}
            /><button class="update-button" onclick={() => finalUpload?.click()}
              >↥ Upload Schedule</button
            ><select
              aria-label="Finals semester"
              bind:value={term}
              onchange={loadFinals}
              >{#each terms as t}<option value={t.id}>{t.label}</option
                >{/each}</select
            ><button
              disabled={!finals.length}
              onclick={async () => {
                try {
                  await data.clearFinals(term);
                  finals = [];
                  message = "Imported exams removed.";
                } catch (cause) {
                  error = reason(cause);
                }
              }}>Remove import</button
            ><button
              class="outline-button"
              disabled={!finals.length}
              onclick={exportFinals}>↓ SIO Export</button
            >
          </div>
          <div class="search-line">
            <img src={searchIcon} alt="" /><input
              aria-label="Search finals"
              placeholder="Search by course ID, name or keyword..."
              bind:value={finalsQuery}
            />
          </div>
          <small>{visibleFinals.length} results</small>
        </div>
        {#if error}<div class="status-panel">{error}</div>{:else}<div
            class="finals-table"
          >
            <div class="finals-row finals-head">
              <span>Course</span><span>Day</span><span>Start Time</span><span
                >End Time</span
              ><span>Location</span>
            </div>
            {#each visibleFinals as exam}<div class="finals-row">
                <strong>{exam.course_code} {exam.course_name}</strong><span
                  >{new Date(exam.starts_at).toLocaleDateString("en-US", {
                    weekday: "long",
                    month: "short",
                    day: "numeric",
                  })}</span
                ><span
                  >{new Date(exam.starts_at).toLocaleTimeString("en-US", {
                    hour: "numeric",
                    minute: "2-digit",
                  })}</span
                ><span
                  >{new Date(exam.ends_at).toLocaleTimeString("en-US", {
                    hour: "numeric",
                    minute: "2-digit",
                  })}</span
                ><span>{exam.location || "TBA"}</span>
              </div>{/each}
          </div>{/if}
      </div>
    {:else if view === "saved" || view === "profile"}
      <aside class="filter-rail profile-rail">
        <div class="rail-heading">
          <h2>Profile</h2>
          <button onclick={() => (filtersOpen = !filtersOpen)}>Hide ‹</button>
        </div>
        <div class="profile-identity">
          <div class="profile-initial">{user?.name?.[0] ?? "?"}</div>
          <div>
            <strong>{user?.name ?? "CMU Student"}</strong><small
              >{user ? "Carnegie Mellon University" : "Sign in with CMU"}</small
            >
          </div>
        </div>
        <nav class="profile-menu" aria-label="Profile sections">
          <button
            class:active={profileSection === "saved"}
            onclick={() => (profileSection = "saved")}
            ><img src={profileSavedIcon} alt="" /> <span>Saved Courses</span> ›</button
          ><button
            class:active={profileSection === "taking"}
            onclick={() => (profileSection = "taking")}
            ><img src={profileTakingIcon} alt="" />
            <span>Currently Taking</span> ›</button
          ><button
            class:active={profileSection === "taken"}
            onclick={() => (profileSection = "taken")}
            ><img src={profileTakenIcon} alt="" /> <span>Courses Taken</span> ›</button
          ><button onclick={() => go("schedule")}
            ><img src={profileScheduleIcon} alt="" /> <span>Schedule</span> ›</button
          ><button
            class:active={profileSection === "friends"}
            onclick={() => {
              profileSection = "friends";
              loadFriends();
            }}
            ><img src={profileFriendsIcon} alt="" /> <span>Friends</span> ›</button
          >
        </nav>
        {#if !user}<a class="profile-login" href="/auth/login"
            >Sign in with CMU</a
          >{/if}
      </aside>
      <div class="work-area profile-work">
        <div class="profile-content-header">
          <h1>
            {profileSection === "saved"
              ? "Saved Courses"
              : profileSection === "taking"
                ? "Currently Taking"
                : profileSection === "taken"
                  ? "Courses Taken"
                  : "Friends"}
          </h1>
          {#if profileSection !== "friends"}<div class="profile-filter-row">
              <select aria-label="Department" bind:value={department}
                ><option value="">Department</option><option
                  value="Computer Science">Computer Science</option
                ></select
              ><select aria-label="Course level" bind:value={level}
                ><option value="">Course Level</option><option value="100"
                  >100 level</option
                ><option value="200">200 level</option><option value="300"
                  >300 level</option
                ><option value="400">400 level</option></select
              >
            </div>{/if}
        </div>
        <div class="profile-content">
          {#if profileSection === "taking" || profileSection === "taken"}<CourseHistory
              status={profileSection}
              {terms}
              {user}
              {openCourse}
            />{:else if profileSection === "friends"}<Friends
              {user}
            />{:else}{#each savedCourses as course}<button
                class="saved-card"
                onclick={() => openCourse(course.code)}
                >{course.code} {course.name}</button
              >{:else}<p>
                Star courses in the catalog to save them here.
              </p>{/each}{/if}
        </div>
      </div>
      <div class="mobile-profile">
        <div class="mobile-profile-section">
          <button
            class:active={profileSection === "saved"}
            onclick={() => (profileSection = "saved")}
            ><img src={profileSavedIcon} alt="" />Saved Courses
            <span>{profileSection === "saved" ? "⌃" : "⌄"}</span></button
          >{#if profileSection === "saved"}<div class="mobile-profile-content">
              <div class="mobile-profile-filters">
                <select aria-label="Department" bind:value={department}
                  ><option value="">Department</option><option
                    value="Computer Science">Computer Science</option
                  ></select
                ><select aria-label="Course level" bind:value={level}
                  ><option value="">Course Level</option><option value="100"
                    >100 level</option
                  ><option value="200">200 level</option><option value="300"
                    >300 level</option
                  ><option value="400">400 level</option></select
                >
              </div>
              {#if savedCourses.length}{#each savedCourses.filter((course) => (!department || course.department === department) && (!level || course.code.startsWith(level[0]))) as course}<button
                    class="mobile-saved-card"
                    onclick={() => openCourse(course.code)}
                    ><strong>{course.code} ↗</strong><span>{course.name}</span
                    ><small>{course.department}</small>
                    <div class="mobile-saved-avatars">
                      <i>C</i><i>A</i><small>+2</small>
                    </div>
                    <div class="mobile-saved-meta">
                      ◔ {course.units} units <span>|</span> ◷ {course.fce?.hours_per_week?.toFixed(
                        1,
                      ) ?? "—"} hrs/week
                    </div></button
                  >{/each}{:else}<p class="mobile-profile-empty">
                  Star courses in the catalog to save them here.
                </p>{/if}
            </div>{/if}
        </div>
        <div class="mobile-profile-section">
          <button
            class:active={profileSection === "taking"}
            onclick={() => (profileSection = "taking")}
            ><img src={profileTakingIcon} alt="" />Currently Taking
            <span>{profileSection === "taking" ? "⌃" : "⌄"}</span></button
          >{#if profileSection === "taking"}<div class="mobile-profile-content">
              <CourseHistory status="taking" {terms} {user} {openCourse} />
            </div>{/if}
        </div>
        <div class="mobile-profile-section">
          <button
            class:active={profileSection === "taken"}
            onclick={() => (profileSection = "taken")}
            ><img src={profileTakenIcon} alt="" />Courses Taken
            <span>{profileSection === "taken" ? "⌃" : "⌄"}</span></button
          >{#if profileSection === "taken"}<div class="mobile-profile-content">
              <CourseHistory status="taken" {terms} {user} {openCourse} />
            </div>{/if}
        </div>
        <div class="mobile-profile-section">
          <button onclick={() => go("schedule")}
            ><img src={profileScheduleIcon} alt="" />Schedule
            <span>⌄</span></button
          >
        </div>
        <div class="mobile-profile-section">
          <button
            class:active={profileSection === "friends"}
            onclick={() => {
              profileSection = "friends";
              loadFriends();
            }}
            ><img src={profileFriendsIcon} alt="" />Friends
            <span>{profileSection === "friends" ? "⌃" : "⌄"}</span></button
          >{#if profileSection === "friends"}<div
              class="mobile-profile-content"
            >
              <Friends {user} />
            </div>{/if}
        </div>
      </div>
    {:else if view === "instructors"}
      <aside class="filter-rail instructor-rail">
        <div class="rail-heading">
          <h2>Filter by</h2>
          <button onclick={() => (filtersOpen = !filtersOpen)}>Hide ‹</button>
        </div>
        <div class="aggregate">
          <h2>Aggregate FCEs</h2>
          <label
            >Semesters to show <select aria-label="Instructor FCE semesters"
              ><option>None</option><option>Most recent</option><option
                >All</option
              ></select
            ></label
          >
          <div class="checks">
            <label><input type="checkbox" checked />Spring</label><label
              ><input type="checkbox" />Summer</label
            ><label><input type="checkbox" checked />Fall</label>
          </div>
        </div>
      </aside>
      <div class="work-area">
        <div class="catalog-search instructor-search">
          <div class="search-line">
            <img src={searchIcon} alt="" /><input
              aria-label="Search instructors"
              placeholder="Search professors by ID, description, name or keyword..."
              bind:value={instructorQuery}
              oninput={() => {
                clearTimeout(timer);
                timer = setTimeout(loadInstructors, 250);
              }}
            />
          </div>
          <div class="search-options">
            <div>
              <span>{instructors.length} results</span>
              <div class="active-filters">
                {#if instructorQuery}<button
                    onclick={() => {
                      instructorQuery = "";
                      loadInstructors();
                    }}>{instructorQuery} ×</button
                  >{/if}
              </div>
            </div>
            <select aria-label="Show instructor results"
              ><option>Show</option></select
            >
          </div>
        </div>
        <div class="instructor-results">
          {#if error}<div class="status-panel">
              <h3>Instructor directory unavailable</h3>
              <p>{error}</p>
              <button onclick={loadInstructors}>Try again</button>
            </div>{:else if !instructors.length}<div class="status-panel">
              No instructors found.
            </div>{:else}{#each instructors as instructor}<article
                class="instructor-card"
              >
                <div class="instructor-summary">
                  <button
                    onclick={async () => {
                      selectedInstructor = instructor;
                      try {
                        selectedInstructor = await data.instructor(
                          instructor.id,
                        );
                      } catch {}
                    }}
                    ><strong>{instructor.name}</strong><small
                      >{instructor.department ??
                        instructor.title ??
                        "Instructor"}</small
                    ></button
                  >
                  <div class="instructor-stat">
                    <strong
                      >{instructor.fce?.hours_per_week?.toFixed(2) ?? "—"}
                      <small>hrs/wk</small></strong
                    ><span>Workload</span>
                  </div>
                  <div class="instructor-stat">
                    <strong
                      >★★★★★ {instructor.fce?.overall_teaching?.toFixed(2) ??
                        "—"}</strong
                    ><span>Teaching Rate</span>
                  </div>
                  <div class="instructor-stat">
                    <strong
                      >★★★★★ {instructor.fce?.overall_course?.toFixed(2) ??
                        "—"}</strong
                    ><span>Course Rate</span>
                  </div>
                </div>
                <div class="instructor-data">
                  <h3>
                    Aggregate Data (data from {instructor.fce?.semesters ?? 0} semesters)
                  </h3>
                  <div class="instructor-data-head">
                    <span>Semester</span><span>Course</span><span>Workload</span
                    ><span>Teaching</span><span>Course Rate</span><span
                      >Respondents</span
                    ><span>Response Rate</span>
                  </div>
                  {#each instructor.courses.slice(0, 4) as code}<div
                      class="instructor-data-row"
                    >
                      <span>—</span><span>{code}</span><span
                        >{instructor.fce?.hours_per_week?.toFixed(2) ??
                          "—"}</span
                      ><span
                        >{instructor.fce?.overall_teaching?.toFixed(2) ??
                          "—"}</span
                      ><span
                        >{instructor.fce?.overall_course?.toFixed(2) ??
                          "—"}</span
                      ><span>—</span><span>—</span>
                    </div>{/each}
                </div>
              </article>{/each}{/if}
        </div>
      </div>
    {:else if view === "geneds"}
      <div class="simple-page">
        <h1>GenEds</h1>
        <div class="search-line">
          <img src={searchIcon} alt="" /><input
            aria-label="Search requirements"
            placeholder="Search GenEd requirements..."
            bind:value={genedQuery}
          />
        </div>
        {#if error}<div class="status-panel">{error}</div>{:else}<div
            class="simple-list"
          >
            {#each visibleGeneds as gened}<button
                onclick={() => {
                  query = gened.name;
                  go("courses");
                  queueCourses();
                }}
                ><strong>{gened.name}</strong><span>{gened.description}</span
                ><small>{gened.courses.length} courses</small></button
              >{/each}
          </div>{/if}
      </div>
    {/if}
  </div>
  <footer class="site-footer">
    <div>
      Designed, developed and maintained with ❤️ by <a
        href="https://scottylabs.org"
        target="_blank"
        rel="noreferrer">ScottyLabs</a
      >.<br />Want to help us make CMU Courses better? Join us at ScottyLabs or
      make a pull request on our
      <a
        href="https://github.com/Scottylabs/cmucourses"
        target="_blank"
        rel="noreferrer">GitHub repository</a
      >.
    </div>
    <a
      class="feedback"
      href="mailto:hello@scottylabs.org?subject=CMU%20Courses%20feedback"
      ><img src={feedbackIcon} alt="" />Feedback</a
    >
  </footer>
</div>
{#if selectedCourse}<div
    class="scrim"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) selectedCourse = null;
    }}
  >
    <div
      class="drawer"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-label={`${selectedCourse.code} details`}
    >
      <div class="drawer-top">
        <span class="pill blue">{selectedCourse.code}</span><button
          class="small-icon"
          aria-label="Close course details"
          onclick={() => (selectedCourse = null)}>×</button
        >
      </div>
      <h2>{selectedCourse.name}</h2>
      <p>{selectedCourse.description}</p>
      {#if detailError}<p class="notice">{detailError}</p>{/if}
      <div class="facts">
        <div><small>Units</small><strong>{selectedCourse.units}</strong></div>
        <div>
          <small>Department</small><strong>{selectedCourse.department}</strong>
        </div>
        <div>
          <small>FCE rating</small><strong
            >{selectedCourse.fce?.overall_course?.toFixed(1) ?? "—"}</strong
          >
        </div>
        <div>
          <small>Hours per week</small><strong
            >{selectedCourse.fce?.hours_per_week?.toFixed(1) ?? "—"}</strong
          >
        </div>
      </div>
      {#if selectedCourse.prerequisites}<h3>Prerequisites</h3>
        <p>
          {selectedCourse.prerequisites}
        </p>{/if}{#if selectedCourse.instructors?.length}<h3>Instructors</h3>
        <p>
          {selectedCourse.instructors.join(", ")}
        </p>{/if}{#if selectedCourse.sections?.length}<h3>Sections</h3>
        {#each selectedCourse.sections as section}<label class="section-choice"
            ><input
              type="checkbox"
              checked={draft.items
                .find((item) => item.course_code === selectedCourse?.code)
                ?.section_ids.includes(section.id) ?? false}
              onchange={(event) =>
                selectedCourse &&
                toggleSection(
                  selectedCourse.code,
                  section.id,
                  event.currentTarget.checked,
                )}
            /><span
              ><strong>{section.label} · {section.kind}</strong><small
                >{section.meetings
                  .map(
                    (meeting) =>
                      `${meeting.days.join("")} ${meeting.starts_at}–${meeting.ends_at}`,
                  )
                  .join(" · ") || "Time TBA"}</small
              ></span
            ></label
          >{/each}{/if}{#if selectedCourse.syllabi?.length}<h3>Syllabi</h3>
        {#each selectedCourse.syllabi as syllabus}<a
            class="syllabus"
            href={syllabus.url}
            target="_blank"
            rel="noreferrer">{syllabus.label} ↗</a
          >{/each}{/if}<button
        class="primary drawer-action"
        onclick={() => {
          if (selectedCourse) addCourse(selectedCourse);
          selectedCourse = null;
          go("schedule");
        }}
        >{selectedCodes.includes(selectedCourse.code)
          ? "View in schedule"
          : "+ Add to schedule"}</button
      >
    </div>
  </div>{/if}
{#if selectedInstructor}<div
    class="scrim"
    role="presentation"
    onclick={(event) => {
      if (event.target === event.currentTarget) selectedInstructor = null;
    }}
  >
    <div
      class="drawer"
      role="dialog"
      tabindex="-1"
      aria-modal="true"
      aria-label={`${selectedInstructor.name} details`}
    >
      <div class="drawer-top">
        <span class="person-avatar">{selectedInstructor.name[0]}</span><button
          class="small-icon"
          aria-label="Close instructor details"
          onclick={() => (selectedInstructor = null)}>×</button
        >
      </div>
      <h2>{selectedInstructor.name}</h2>
      <p>{selectedInstructor.title ?? selectedInstructor.department}</p>
      {#if selectedInstructor.email}<a
          href={`mailto:${selectedInstructor.email}`}
          >{selectedInstructor.email}</a
        >{/if}
      <h3>Courses taught</h3>
      <div class="tags">
        {#each selectedInstructor.courses as code}<button
            onclick={() => {
              selectedInstructor = null;
              openCourse(code);
            }}>{code}</button
          >{/each}
      </div>
      {#if selectedInstructor.fce}<h3>Aggregate FCEs</h3>
        <p>
          Course: {selectedInstructor.fce.overall_course?.toFixed(1) ?? "—"} · Teaching:
          {selectedInstructor.fce.overall_teaching?.toFixed(1) ?? "—"}
        </p>{/if}
    </div>
  </div>{/if}
