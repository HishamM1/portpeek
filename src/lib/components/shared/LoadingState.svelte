<script lang="ts">
  import LoaderCircle from "@lucide/svelte/icons/loader-circle";
</script>

<div class="grid flex-1 place-items-center" role="status">
  <div class="flex items-center gap-2 text-xs text-[var(--text-secondary)]">
    <LoaderCircle size={15} strokeWidth={1.8} class="animate-spin" />
    Scanning local ports
  </div>
</div>
