<script lang="ts">
  // Where to download the PDFs the import reads (Courseweave's HowTo in features/record/SioImport.tsx).
  let { class: className = "" }: { class?: string } = $props();
</script>

<p class="text-body-2 text-muted {className}">
  In SIO, open <span class="text-ink">Academic Record</span> and select <span class="text-ink">PDF</span> under
  Unofficial Academic Record. A Stellic plan works too, and also becomes a degree plan: in
  <span class="text-ink">Plan your Path</span>, select <span class="text-ink">Download Plan</span>, then
  <span class="text-ink">Create plan report</span>. The file is read on this device and never uploaded.
</p>
