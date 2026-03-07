import es from './locales/es';
import en from './locales/en';
import type { Translations } from './types';

const dicts: Record<string, Translations> = { es, en };

let _locale = $state<string>(
	(typeof localStorage !== 'undefined' && localStorage.getItem('dedaliano-lang')) || 'es'
);

export function t(key: string): string {
	const dict = dicts[_locale] ?? dicts.es;
	return (dict as any)[key] ?? (dicts.es as any)[key] ?? key;
}

export function setLocale(loc: string) {
	_locale = loc;
	if (typeof localStorage !== 'undefined') {
		localStorage.setItem('dedaliano-lang', loc);
	}
}

/** Set of all translations for a given key (across every locale). */
function allTranslations(key: string): Set<string> {
	const s = new Set<string>();
	for (const dict of Object.values(dicts)) {
		const v = (dict as any)[key];
		if (v) s.add(v);
	}
	return s;
}

/** Returns true if `name` matches any locale's default structure name. */
export function isDefaultName(name: string): boolean {
	return allTranslations('tabBar.newStructure').has(name);
}

export const i18n = {
	get locale() {
		return _locale;
	},
	set locale(v: string) {
		setLocale(v);
	},
	t,
	setLocale
};
