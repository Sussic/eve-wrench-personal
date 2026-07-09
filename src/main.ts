import { createApp } from 'vue'
import App from './App.vue'
import './style.css'
import i18n from './locales'
import router from './router'

createApp(App).use(i18n).use(router).mount('#app')
