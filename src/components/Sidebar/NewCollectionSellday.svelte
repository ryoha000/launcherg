<script lang="ts">
  import NewCollectionOption from "@/components/Sidebar/NewCollectionOption.svelte";
  import NewCollectionSelldayInput from "@/components/Sidebar/NewCollectionSelldayInput.svelte";
  import { fly } from "svelte/transition";

  export let isFilterSellday: boolean;
  export let filterSellday: {
    since: string | undefined;
    until: string | undefined;
  };

  let sinceYear = "";
  let sinceMonth = "";

  let untilYear = "";
  let untilMonth = "";

  $: {
    filterSellday = {
      since: convertValid(
        { year: sinceYear, month: sinceMonth },
        { year: START_YEAR, month: START_MONTH, day: () => START_DAY }
      ),
      until: convertValid(
        { year: untilYear, month: untilMonth },
        { year: END_YEAR, month: END_MONTH, day: getEndDayByMonth }
      ),
    };
    console.log({ filterSellday });
  }

  const START_YEAR = 1970;
  const START_MONTH = 1;
  const START_DAY = 1;

  const END_YEAR = 2050;
  const END_MONTH = 12;
  const pad = (num: number, max = 2) => `${num}`.padStart(max, "0");
  const getEndDayByMonth = (month: number) => {
    if ([2, 4, 6, 9, 11].includes(month)) {
      if (2) {
        return 28;
      }
      return 30;
    }
    return 31;
  };
  const convertValid = (
    date: { year: string; month: string },
    defaultDate: { year: number; month: number; day: (month: number) => number }
  ) => {
    const inputYear = +date.year.replace(/[^0-9\-]/g, "");
    const inputMonth = +date.month.replace(/[^0-9\-]/g, "");

    const inputs = [inputYear, inputMonth];

    if (inputs.some((input) => isNaN(input))) {
      console.error("Invalid date string", inputs);
      return;
    }
    if ((inputYear && inputYear < 1970) || inputYear > 2050) {
      console.error("year is invalid", inputs);
      return;
    }
    if ((inputMonth && inputMonth < 1) || inputMonth > 12) {
      console.error("month is invalid", inputs);
      return;
    }

    if (!inputYear && inputMonth) {
      console.error("year is 0 but month is not 0", inputs);
      return;
    }

    let year = defaultDate.year;
    if (inputYear) {
      year = inputYear;
    }

    let month = defaultDate.month;
    if (inputMonth) {
      month = inputMonth;
    }

    const day = defaultDate.day(month);

    return `${pad(year, 4)}-${pad(month)}-${pad(day)}`;
  };
</script>

<div class="space-y-2">
  <NewCollectionOption
    bind:value={isFilterSellday}
    label="範囲内で発売された"
  />
  {#if isFilterSellday}
    <div class="pl-8 space-y-2" transition:fly={{ y: -40, duration: 150 }}>
      <NewCollectionSelldayInput
        label="絞り込み開始"
        bind:year={sinceYear}
        bind:month={sinceMonth}
      />
      <NewCollectionSelldayInput
        label="絞り込み終了"
        bind:year={untilYear}
        bind:month={untilMonth}
      />
    </div>
  {/if}
</div>
