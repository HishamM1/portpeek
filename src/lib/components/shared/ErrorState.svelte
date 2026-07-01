<script lang="ts">
  import CircleAlert from "@lucide/svelte/icons/circle-alert";

  let {
    message,
    onRetry,
  }: {
    message: string;
    onRetry: () => void | Promise<void>;
  } = $props();
</script>

<div class="grid flex-1 place-items-center px-6 text-center" role="alert">
  <div>
    <CircleAlert size={20} strokeWidth={1.7} class="mx-auto text-[var(--danger)]" />
    <p class="mt-2 text-xs text-[var(--text-secondary)]">{message}</p>
    <button type="button" class="mt-3 text-xs font-semibold text-[var(--primary)]" onclick={onRetry}>
      Try again
    </button>
  </div>
</div>
