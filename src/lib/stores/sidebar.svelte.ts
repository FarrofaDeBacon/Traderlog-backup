export const sidebarState = $state({
    isCollapsed: false,
    toggle() {
        this.isCollapsed = !this.isCollapsed;
    }
});
