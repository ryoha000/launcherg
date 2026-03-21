<script lang='ts'>
  import type { VoiceActor, Work } from '@/lib/types'
  import LinkText from '@/components/UI/LinkText.svelte'
  import DetailRow from '@/components/Work/DetailRow.svelte'
  import {

    VoiceActorImportance,

  } from '@/lib/types'

  interface Props {
    work: Work
  }

  const { work }: Props = $props()

  const getCreatorUrl = (id: number) =>
    `https://erogamescape.dyndns.org/~ap2/ero/toukei_kaiseki/creater.php?creater=${id}`
  const getVoiceActorClass = (importance: VoiceActor['importance']) => {
    switch (importance) {
      case VoiceActorImportance.Main:
        return 'text-(body3 text-secondary) font-bold'
      case VoiceActorImportance.Sub:
        return 'text-(body3 text-primary)'
      case VoiceActorImportance.Mob:
        return 'text-(body3 text-tertiary)'
      default:
        throw new Error(`Unknown importance: ${importance satisfies never}`)
    }
  }
</script>

<div class='border border-(border-primary solid) rounded-xl'>
  <div class='grid grid-(cols-[min-content_1fr])'>
    <DetailRow label='シナリオ' noBorder>
      <div class='flex flex-wrap gap-4'>
        {#each work.creators.writers as v (v.id)}
          <LinkText href={getCreatorUrl(v.id)} text={v.name} />
        {/each}
      </div>
    </DetailRow>
    {#if work.creators.illustrators.length}
      <DetailRow label='原画'>
        <div class='flex flex-wrap gap-4'>
          {#each work.creators.illustrators as v (v.id)}
            <LinkText href={getCreatorUrl(v.id)} text={v.name} />
          {/each}
        </div>
      </DetailRow>
    {/if}
    {#if work.creators.voiceActors.length}
      <DetailRow label='声優'>
        <div class='flex flex-wrap gap-4'>
          {#each work.creators.voiceActors as v (v.id)}
            <div class='flex items-end gap-1'>
              <LinkText href={getCreatorUrl(v.id)} text={v.name} />
              <div class={getVoiceActorClass(v.importance)}>{v.role}</div>
            </div>
          {/each}
        </div>
      </DetailRow>
    {/if}
    {#if work.musics.length}
      <DetailRow label='楽曲'>
        <div class='flex flex-wrap gap-4'>
          {#each work.musics as title, i (`${title}-${i}`)}
            <div class='max-w-full flex items-center gap-1'>
              <LinkText
                href={encodeURI(
                  `https://www.youtube.com/results?search_query=${work.name}+${title}`,
                )}
              >
                <div class='flex items-center gap-1'>
                  <div class='i-iconoir-youtube h-4 w-4 color-#cc0000'></div>
                  {title}
                </div>
              </LinkText>
            </div>
          {/each}
        </div>
      </DetailRow>
    {/if}
  </div>
</div>
