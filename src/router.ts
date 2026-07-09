import { createRouter, createWebHashHistory } from 'vue-router'
import MainView from './views/MainView.vue'

// Secondary windows navigate here via hash URLs built on the Rust side,
// e.g. index.html#/formations?path=...&name=...
const router = createRouter({
    history: createWebHashHistory(),
    routes: [
        { path: '/', component: MainView },
        {
            path: '/formations',
            component: () => import('./views/FormationEditorView.vue'),
            props: (route) => ({
                filePath: String(route.query.path ?? ''),
                entryName: String(route.query.name ?? ''),
            }),
        },
    ],
})

export default router
