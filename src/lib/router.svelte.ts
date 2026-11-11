/**
 * 是一个设备工具，页面层级很浅：五个一级页面，没有详情页。
 * 所以路由器只保留「当前页 + 上一页」，不做复杂的返回栈。
 */
export type RouteName = 'overview' | 'dpi' | 'buttons' | 'performance' | 'settings';

export const router = $state<{ route: RouteName; previous: RouteName | null }>({
  route: 'overview',
  previous: null,
});

/** 跨页面的小交互状态：从概览点鼠标预览时，把按键页带到对应行。 */
export const uiState = $state<{ buttonFocus: number }>({ buttonFocus: -1 });

export function go(name: RouteName) {
  if (router.route === name) {
    scrollMainTop();
    return;
  }
  router.previous = router.route;
  router.route = name;
  scrollMainTop();
}

/** 切页后把主滚动区域拉回顶部，和播放器/设置页的手感一致。 */
export function scrollMainTop() {
  requestAnimationFrame(() => {
    document.querySelector('main')?.scrollTo({ top: 0, behavior: 'auto' });
  });
}
