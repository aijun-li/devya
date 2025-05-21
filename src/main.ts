import { VueQueryPlugin } from '@tanstack/vue-query';
import { createPinia } from 'pinia';
import PrimeVue from 'primevue/config';
import { createApp } from 'vue';
import { createRouter, createWebHistory } from 'vue-router';
import { routes } from 'vue-router/auto-routes';
import App from './App.vue';
import './assets/base.css';
import './assets/prime.scss';
import './assets/tailwind.css';
import { Noir } from './theme';

const app = createApp(App);

app.use(PrimeVue, {
  theme: {
    preset: Noir,
  },
});

app.use(VueQueryPlugin);

const router = createRouter({ history: createWebHistory(), routes });
app.use(router);

const pinia = createPinia();
app.use(pinia);

app.mount('#app');
