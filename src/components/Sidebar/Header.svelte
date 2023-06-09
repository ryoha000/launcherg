<script lang="ts">
  import Logo from "/logo.png";
  import Icon from "/icon.png";
  import { link } from "svelte-spa-router";
  import { showSidebar } from "@/store/showSidebar";
  import ButtonBase from "@/components/UI/ButtonBase.svelte";
</script>

<div class="w-full flex items-center px-2 pt-4">
  <a href="/" use:link>
    <div class="flex items-center gap-2 w-full">
      <img src={Icon} alt="launcherg icon" class="h-8" />
      <img src={Logo} alt="launcherg logo" class="h-7" />
    </div>
  </a>
  <ButtonBase
    on:click={() => showSidebar.set(false)}
    appendClass="ml-auto border-0px p-1 bg-transparent"
    tooltip={{
      content: "サイドバーを閉じる",
      placement: "bottom",
      theme: "default",
      delay: 1000,
    }}
  >
    <div
      class="i-material-symbols-left-panel-close-outline w-6 h-6 color-text-primary"
    />
  </ButtonBase>
</div>
