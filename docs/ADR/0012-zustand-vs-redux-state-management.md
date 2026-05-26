# ADR 0012: Zustand vs Redux vs Context API for Frontend State Management

**Status**: ✅ **Accepted**  
**Date**: 2026-05-20  
**Deciders**: Claude Code (Frontend Engineer), AjDRoger (UX Lead)  
**Context**: Fase 4 React Frontend, global state management for multi-workspace RAG UI

---

## Context

### Problema

Archivio Parlante frontend (React 18 + TypeScript) needs **global state management** for:

1. **Authentication**: User session, JWT tokens, refresh logic
2. **Workspace Context**: Current workspace_id, role (owner/admin/member)
3. **Chat State**: Active conversation, message history, streaming LLM responses
4. **Document Browser**: Selected KB, document filters, pagination state
5. **Comparison Matrix**: Selected contracts (2-10), aspect selection
6. **UI State**: Sidebar collapsed, theme (light/dark), loading indicators

**Requirements**:
- Type-safe (TypeScript strict mode)
- Minimal boilerplate (developer velocity)
- Devtools support (debugging)
- React 18 Concurrent Mode compatible
- SSR-ready (future Vite SSR support)
- Bundle size < 50KB (including state management)

**Non-Requirements**:
- ❌ Time-travel debugging (not needed for legal app)
- ❌ Redux DevTools (nice-to-have, not critical)
- ❌ Middleware ecosystem (server state handled by react-query)

---

## Decision Drivers

| Factor | Weight | Notes |
|---|---|---|
| **Developer Experience** | 🔴 CRITICAL | Minimal boilerplate, TypeScript inference |
| **Bundle Size** | 🟡 HIGH | Target < 10KB for state lib |
| **Learning Curve** | 🟡 HIGH | Team new to project, onboarding speed matters |
| **Performance** | 🟢 MEDIUM | Re-renders matter, but app not render-heavy |
| **Ecosystem** | 🟢 LOW | react-query handles server state |

---

## Options Considered

### Option A: Zustand
**Status**: ✅ **ACCEPTED**

```typescript
// stores/authStore.ts
import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface AuthState {
  user: User | null;
  accessToken: string | null;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      
      login: async (email, password) => {
        const response = await fetch('/api/auth/login', {
          method: 'POST',
          body: JSON.stringify({ email, password }),
        });
        const { user, access_token } = await response.json();
        set({ user, accessToken: access_token });
      },
      
      logout: () => {
        set({ user: null, accessToken: null });
      },
      
      refreshToken: async () => {
        // Refresh logic...
      },
    }),
    { name: 'auth-storage' } // Persist to localStorage
  )
);

// Usage in component
function Header() {
  const user = useAuthStore((state) => state.user); // Selector (auto-optimized)
  const logout = useAuthStore((state) => state.logout);
  
  return (
    <button onClick={logout}>
      Logout {user?.email}
    </button>
  );
}
```

**Pros**:
- ✅ **Minimal Boilerplate**: 10 lines vs 50 lines Redux
- ✅ **TypeScript First**: Full inference, no manual typing
- ✅ **Tiny Bundle**: 1.8KB gzipped (Redux: 3KB + 2KB Toolkit)
- ✅ **No Provider Hell**: Direct hook usage, no `<Provider>` wrapper
- ✅ **Auto-Optimized**: Selector-based subscriptions (no manual memoization)
- ✅ **Middleware**: persist, devtools, immer built-in
- ✅ **Concurrent Mode**: React 18 compatible out-of-box
- ✅ **SSR-Ready**: No hydration issues

**Cons**:
- ⚠️ Less mature ecosystem than Redux (but sufficient for 95% cases)
- ⚠️ No time-travel debugging (acceptable trade-off)

**Bundle Size**:
```
zustand:          1.8KB gzipped
zustand/persist:  +0.5KB
zustand/immer:    +0.3KB (if needed)
Total:            2.6KB
```

---

### Option B: Redux Toolkit (RTK)
**Status**: ❌ **Rejected** (boilerplate overhead)

```typescript
// store.ts
import { configureStore, createSlice, PayloadAction } from '@reduxjs/toolkit';

const authSlice = createSlice({
  name: 'auth',
  initialState: { user: null, accessToken: null } as AuthState,
  reducers: {
    setUser: (state, action: PayloadAction<User>) => {
      state.user = action.payload;
    },
    setAccessToken: (state, action: PayloadAction<string>) => {
      state.accessToken = action.payload;
    },
    logout: (state) => {
      state.user = null;
      state.accessToken = null;
    },
  },
});

export const { setUser, setAccessToken, logout } = authSlice.actions;

// Thunk for async logic
export const login = (email: string, password: string) => async (dispatch) => {
  const response = await fetch('/api/auth/login', { ... });
  const { user, access_token } = await response.json();
  dispatch(setUser(user));
  dispatch(setAccessToken(access_token));
};

export const store = configureStore({
  reducer: { auth: authSlice.reducer },
});

// Type boilerplate
export type RootState = ReturnType<typeof store.getState>;
export type AppDispatch = typeof store.dispatch;

// Typed hooks
export const useAppDispatch = () => useDispatch<AppDispatch>();
export const useAppSelector: TypedUseSelectorHook<RootState> = useSelector;

// Usage in component (requires Provider)
import { Provider } from 'react-redux';

function App() {
  return (
    <Provider store={store}>
      <Header />
    </Provider>
  );
}

function Header() {
  const user = useAppSelector((state) => state.auth.user);
  const dispatch = useAppDispatch();
  
  return (
    <button onClick={() => dispatch(logout())}>
      Logout {user?.email}
    </button>
  );
}
```

**Pros**:
- ✅ Mature ecosystem (10 years)
- ✅ Excellent DevTools (time-travel, state inspector)
- ✅ Immer built-in (mutable updates)
- ✅ Familiar to many devs

**Cons**:
- ❌ **BLOCKER**: 5x more boilerplate than Zustand (50 lines vs 10)
- ❌ **BLOCKER**: Type inference requires manual setup (RootState, AppDispatch, typed hooks)
- ❌ Larger bundle: 5KB (Redux 3KB + RTK 2KB) vs 1.8KB Zustand
- ❌ Provider wrapper required (boilerplate in App.tsx)
- ❌ Actions/reducers separation (cognitive overhead)
- ❌ Async requires thunks (extra middleware)

**Why NOT Redux Toolkit**:
- We don't need time-travel debugging
- We use react-query for server state (Redux overkill)
- 5x boilerplate = slower development velocity
- Team not familiar with Redux (learning curve)

---

### Option C: React Context + useReducer
**Status**: ❌ **Rejected** (performance issues, boilerplate)

```typescript
// AuthContext.tsx
const AuthContext = createContext<AuthState | undefined>(undefined);

export function AuthProvider({ children }) {
  const [state, dispatch] = useReducer(authReducer, initialState);
  
  return (
    <AuthContext.Provider value={{ state, dispatch }}>
      {children}
    </AuthContext.Provider>
  );
}

export function useAuth() {
  const context = useContext(AuthContext);
  if (!context) throw new Error('useAuth must be inside AuthProvider');
  return context;
}

// Usage
function App() {
  return (
    <AuthProvider>
      <WorkspaceProvider>
        <ChatProvider>
          <Header />
        </ChatProvider>
      </WorkspaceProvider>
    </AuthProvider>
  );
}
```

**Pros**:
- ✅ No external dependency (React built-in)
- ✅ Simple mental model (Provider/Consumer)

**Cons**:
- ❌ **BLOCKER**: Provider hell (3+ providers = JSX nesting nightmare)
- ❌ **BLOCKER**: Re-render issues (entire subtree re-renders on any state change)
- ❌ No devtools (debugging with console.log)
- ❌ No persistence (manual localStorage)
- ❌ Boilerplate for each context (Provider, hook, types)
- ❌ No selectors (manual memoization with useMemo)

**Performance Issue**:
```typescript
// ❌ BAD: Entire app re-renders when auth.user.email changes
<AuthContext.Provider value={{ user, logout }}>
  <ExpensiveComponent /> {/* Re-renders even if it doesn't use auth */}
</AuthContext.Provider>

// ✅ GOOD: Zustand selector-based (only Header re-renders)
const user = useAuthStore((state) => state.user);
```

---

### Option D: Jotai / Recoil (Atomic State)
**Status**: ❌ **Rejected** (overkill, experimental)

```typescript
// atoms/authAtom.ts
import { atom } from 'jotai';

export const userAtom = atom<User | null>(null);
export const accessTokenAtom = atom<string | null>(null);

// Usage
function Header() {
  const [user, setUser] = useAtom(userAtom);
  return <button onClick={() => setUser(null)}>Logout</button>;
}
```

**Pros**:
- ✅ Fine-grained reactivity (atom-level subscriptions)
- ✅ TypeScript first
- ✅ Small bundle (3KB)

**Cons**:
- ❌ Atom management overhead (100+ atoms for complex app)
- ❌ Less intuitive than store-based approach
- ❌ Smaller ecosystem (Recoil experimental, Jotai young)
- ❌ Overkill for our use case (not rendering 1000s of items)

---

## Decision

**ACCEPTED**: Zustand with persist & devtools middleware

**Rationale**:
1. **Developer Experience**: 10 lines of code vs 50 (Redux) or Provider hell (Context)
2. **TypeScript**: Full inference, no manual type setup
3. **Bundle Size**: 1.8KB vs 5KB Redux (70% smaller)
4. **Performance**: Auto-optimized selectors, no manual memoization
5. **Simplicity**: Direct hook usage, no Provider wrapper
6. **Proven**: Used by Vercel, Resend, Paddle (production apps)

**Implementation**:

```bash
npm install zustand
```

```typescript
// stores/authStore.ts
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';

interface AuthState {
  user: User | null;
  accessToken: string | null;
  isAuthenticated: boolean;
  login: (email: string, password: string) => Promise<void>;
  logout: () => void;
  refreshToken: () => Promise<void>;
}

export const useAuthStore = create<AuthState>()(
  persist(
    (set, get) => ({
      user: null,
      accessToken: null,
      get isAuthenticated() {
        return !!get().accessToken;
      },
      
      login: async (email, password) => {
        const response = await authApi.login(email, password);
        set({ user: response.user, accessToken: response.access_token });
      },
      
      logout: () => {
        set({ user: null, accessToken: null });
        // Clear react-query cache
        queryClient.clear();
      },
      
      refreshToken: async () => {
        const { access_token } = await authApi.refresh();
        set({ accessToken: access_token });
      },
    }),
    {
      name: 'auth-storage',
      storage: createJSONStorage(() => sessionStorage), // In-memory (more secure than localStorage)
      partialize: (state) => ({ accessToken: state.accessToken }), // Only persist token, not user object
    }
  )
);

// Devtools (development only)
if (import.meta.env.DEV) {
  import('zustand/middleware').then(({ devtools }) => {
    useAuthStore = create(devtools(useAuthStore));
  });
}
```

```typescript
// stores/workspaceStore.ts
export const useWorkspaceStore = create<WorkspaceState>((set) => ({
  currentWorkspaceId: null,
  role: null,
  setWorkspace: (id, role) => set({ currentWorkspaceId: id, role }),
  clearWorkspace: () => set({ currentWorkspaceId: null, role: null }),
}));

// stores/chatStore.ts
export const useChatStore = create<ChatState>((set) => ({
  messages: [],
  isStreaming: false,
  addMessage: (msg) => set((state) => ({ messages: [...state.messages, msg] })),
  setStreaming: (streaming) => set({ isStreaming: streaming }),
  clearMessages: () => set({ messages: [] }),
}));
```

**Usage in Components**:
```typescript
// Selector-based (auto-optimized, only re-renders when email changes)
function UserEmail() {
  const email = useAuthStore((state) => state.user?.email);
  return <span>{email}</span>;
}

// Multiple selectors
function Header() {
  const { isAuthenticated, logout } = useAuthStore((state) => ({
    isAuthenticated: state.isAuthenticated,
    logout: state.logout,
  }));
  
  if (!isAuthenticated) return <LoginButton />;
  return <button onClick={logout}>Logout</button>;
}

// Outside React (e.g., axios interceptor)
import { useAuthStore } from './stores/authStore';

axios.interceptors.request.use((config) => {
  const token = useAuthStore.getState().accessToken;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});
```

---

## Consequences

### Positive
- ✅ Fast development: 80% less boilerplate vs Redux
- ✅ TypeScript happy: Full inference, no type gymnastics
- ✅ Tiny bundle: 1.8KB (leaves room for features)
- ✅ Easy testing: `useAuthStore.setState({ ... })` in tests
- ✅ No Provider wrapper: Cleaner App.tsx
- ✅ Concurrent Mode compatible: React 18 ready

### Negative
- ⚠️ Less Redux ecosystem (but we use react-query for server state)
- ⚠️ No time-travel debugging (acceptable for legal app)

### Neutral
- 📌 Performance: Selector-based subscriptions sufficient for our UI (not rendering 10k rows)
- 📌 Learning curve: 1 hour to learn Zustand vs 1 day for Redux

---

## Monitoring & Observability

**State Debugging**:
```typescript
// Development: Zustand DevTools
import { devtools } from 'zustand/middleware';

// Production: Sentry breadcrumbs
useAuthStore.subscribe((state) => {
  Sentry.addBreadcrumb({
    category: 'state',
    message: `Auth state changed: ${state.isAuthenticated}`,
    level: 'info',
  });
});
```

---

## References

- [Zustand Documentation](https://github.com/pmndrs/zustand) - Official docs
- [Why Zustand over Redux](https://tkdodo.eu/blog/working-with-zustand) - TkDodo blog
- [Vercel uses Zustand](https://vercel.com/blog/how-we-optimized-package-imports-in-next-js) - Industry adoption
- [Zustand vs Redux benchmark](https://github.com/dai-shi/will-this-react-global-state-work-in-concurrent-rendering) - Concurrent rendering

---

**Decision Maker**: Claude Sonnet 4.5  
**Approved By**: AjDRoger (implicit via CLAUDE.md §7.4 - Zustand for frontend)  
**Implemented**: `frontend/src/store/` (Fase 4)  
**Review Date**: 2026-07-01 (after 1 month production usage)
