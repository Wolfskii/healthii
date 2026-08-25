# Frontend architecture

Web and desktop share `@healthii/ui`. Mobile reimplements layout with React Native using `@healthii/design-tokens` and `@healthii/dashboard`.

State lives in `SessionProvider` (`@healthii/ui`) talking only to `@healthii/api-client`. Mobile uses the same client with Expo `fetch`.

i18n: keep copy in view components or message catalogs, not in calculation helpers.

Accessibility: skip link, landmarks, labels, focus rings, reduced motion, light/dark themes with AA contrast on paper/ink.
