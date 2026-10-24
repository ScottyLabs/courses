<script lang="ts">
  let {
    value,
  }: {
    /** Page name; the document title becomes "{value} | CMU Courses". Omit for the home page. */
    value?: string;
  } = $props();
</script>

<svelte:head>
  <title>{value ? `${value} | CMU Courses` : "CMU Courses"}</title>
</svelte:head>
