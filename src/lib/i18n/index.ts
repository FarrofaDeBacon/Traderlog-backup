import { register, init, getLocaleFromNavigator, waitLocale } from 'svelte-i18n';
import { browser } from '$app/environment';

export const setupI18n = async () => {
    register('pt-BR', () => import('./locales/pt-BR.json'));
    register('en', () => import('./locales/en-US.json'));
    register('es-ES', () => import('./locales/es-ES.json'));
    register('fr-FR', () => import('./locales/fr-FR.json'));

    const initialLocale = browser
        ? window.localStorage.getItem('locale') || getLocaleFromNavigator() || 'pt-BR'
        : 'pt-BR';

    init({
        fallbackLocale: 'pt-BR',
        initialLocale,
    });

    if (browser) {
        await waitLocale();
    }
};
