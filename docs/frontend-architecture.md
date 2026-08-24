# Frontend architecture

Web and desktop share `@healthii/ui`. Mobile reimplements layout with React Native using `@healthii/design-tokens` and `@healthii/dashboard`.

State stays local in Phase 1. When records exist, prefer React Query (or similar) talking only to `@healthii/api-client`.

i18n: keep copy in view components or message catalogs, not in calculation helpers.

Accessibility: skip link, landmarks, labels, focus rings, reduced motion, light/dark themes with AA contrast on paper/ink.
