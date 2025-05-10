import PrimeVue from 'primevue/config';
import { createApp } from 'vue';
import App from './App.vue';
import './assets/base.css';
import './assets/tailwind.css';

const app = createApp(App);

app.use(PrimeVue, {
  unstyled: true,
});

app.mount('#app');
