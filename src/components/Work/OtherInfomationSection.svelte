<script lang="ts">
  export let label: string;
  export let value: string | number | undefined | null;
</script>

<div class="space-y-2">
  <div class="text-(body2 text-primary) font-medium">{label}</div>
  <div class="text-(body2 text-secondary)">
    {value}
  </div>
</div>
