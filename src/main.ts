import { createPinia } from 'pinia';
import { createApp } from 'vue';
import { createRouter, createWebHashHistory } from 'vue-router';
import { routes } from 'vue-router/auto-routes';
import App from './App.vue';
import './styles/global.css';

const router = createRouter({
  history: createWebHashHistory(),
  routes,
});

const pinia = createPinia();

createApp(App).use(router).use(pinia).mount('#app');
