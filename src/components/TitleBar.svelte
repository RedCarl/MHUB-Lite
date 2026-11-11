<script lang="ts">
  import { HugeiconsIcon } from '@hugeicons/svelte';
  import { icons, ICON_SIZE } from '../lib/icons';
</script>

<!--
  Windows 自绘标题栏。整条都是窗口的「把手」：
  data-tauri-drag-region="deep" 让品牌名和空白处都能拖动窗口。
  右上角的 WindowControls 是固定定位的，不在这一层里。
-->
<header class="titlebar" data-tauri-drag-region="deep">
  <div class="brand">
    <span class="logo"><HugeiconsIcon icon={icons.mouse} size={ICON_SIZE.inline} /></span>
    <span>MHUB Lite</span>
  </div>
</header>
