import { createApp } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import App from './App.vue'
import './style.css'
import './practice.css'
import { retireLegacyClient } from './legacyClient'

const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: '/:pathMatch(.*)*', component: { template: '<span />' } }],
})
void retireLegacyClient().finally(() => createApp(App).use(router).mount('#app'))
