<script lang="ts">
  import { onMount } from "svelte";
  import EntryForm from "./EntryForm.svelte";
  import type { ActivitySuggestion } from "../lib/activitySuggestions";
  import type { RmCatalogCache } from "../lib/rm";

  export let open = false;
  export let project = "";
  export let assignableId: number | null = null;
  export let category = "";
  export let startTime = "";
  export let endTime = "";
  export let notes = "";
  export let error = "";
  export let isSaving = false;
  export let isLoading = false;
  export let isEditing = false;
  export let pendingSuggestion: ActivitySuggestion | null = null;
  export let catalog: RmCatalogCache | null = null;
  export let onSubmit: () => void;
  export let onClose: () => void;

  let dialogEl: HTMLDialogElement;
  let previousFocus: HTMLElement | null = null;

  const handleBackdropClick = (event: MouseEvent) => {
    if (event.target === dialogEl) {
      onClose();
    }
  };

  const handleKeydown = (event: KeyboardEvent) => {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
    }
  };

  $: if (dialogEl) {
    if (open && !dialogEl.open) {
      previousFocus = document.activeElement as HTMLElement;
      dialogEl.showModal();
    } else if (!open && dialogEl.open) {
      dialogEl.close();
      previousFocus?.focus();
      previousFocus = null;
    }
  }

  onMount(() => {
    return () => {
      if (dialogEl?.open) {
        dialogEl.close();
      }
    };
  });
</script>

<dialog
  bind:this={dialogEl}
  class="entry-modal"
  aria-modal="true"
  aria-labelledby="entry-modal-title"
  on:click={handleBackdropClick}
  on:keydown={handleKeydown}
>
  <div class="entry-modal-card">
    <h2 id="entry-modal-title" class="entry-modal-title">
      {isEditing ? "Edit entry" : pendingSuggestion ? "Create entry" : "Add entry"}
    </h2>

    <EntryForm
      bind:project
      bind:assignableId
      bind:category
      bind:startTime
      bind:endTime
      bind:notes
      {error}
      {isSaving}
      {isLoading}
      {isEditing}
      {pendingSuggestion}
      {catalog}
      onSubmit={onSubmit}
      onCancel={onClose}
    />
  </div>
</dialog>
