<script lang="ts">
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
  import { type Props as TippyOption } from "tippy.js";

  interface Props {
    icon: string;
    tooltip?: TippyOption | undefined;
  }

  let { icon, tooltip = undefined }: Props = $props();
</script>

<ButtonBase appendClass="h-8 w-8 flex items-center justify-center" {tooltip}>
  <div class="color-ui-tertiary w-5 h-5 {icon}"></div>
</ButtonBase>
