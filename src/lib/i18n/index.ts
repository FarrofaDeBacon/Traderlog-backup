import { register, init, getLocaleFromNavigator } from 'svelte-i18n';
import { browser } from '$app/environment';

export const setupI18n = () => {
    register('pt-BR', () => import('./locales/pt-BR.json'));
    register('en', () => import('./locales/en-US.json'));
    register('es-ES', () => import('./locales/es-ES.json'));
    register('fr-FR', () => import('./locales/fr-FR.json'));

    init({
        fallbackLocale: 'pt-BR',
        initialLocale: browser
            ? window.localStorage.getItem('locale') || getLocaleFromNavigator()
            : 'pt-BR',
    });
};
