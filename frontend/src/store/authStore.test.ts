import { describe, it, expect, vi, beforeEach } from 'vitest';
import { useAuthStore } from './authStore';
import api from '../lib/api';

// Mock the API client
vi.mock('../lib/api', () => ({
  default: {
    login: vi.fn(),
    register: vi.fn(),
    logout: vi.fn(),
    getCurrentUser: vi.fn(),
  },
}));

describe('authStore', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    localStorage.clear();
    useAuthStore.setState({
      user: null,
      isAuthenticated: false,
      isLoading: false,
      error: null,
    });
  });

  describe('login', () => {
    it('sets user and isAuthenticated on successful login', async () => {
      const mockUser = {
        id: 1,
        email: 'test@example.com',
        full_name: 'Test User',
        role: 'user' as const,
        created_at: '2024-01-01',
      };

      (api.login as any).mockResolvedValue({ access_token: 'token123', refresh_token: 'refresh123' });
      (api.getCurrentUser as any).mockResolvedValue(mockUser);

      const store = useAuthStore.getState();
      await store.login('test@example.com', 'password123');

      const state = useAuthStore.getState();
      expect(state.user).toEqual(mockUser);
      expect(state.isAuthenticated).toBe(true);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBeNull();
    });

    it('sets error on failed login with API error message', async () => {
      const errorResponse = {
        response: {
          data: {
            message: 'Invalid credentials',
          },
        },
      };

      (api.login as any).mockRejectedValue(errorResponse);

      const store = useAuthStore.getState();

      try {
        await store.login('test@example.com', 'wrongpassword');
      } catch (error) {
        // Expected to throw
      }

      const state = useAuthStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(state.isLoading).toBe(false);
      expect(state.error).toBe('Invalid credentials');
    });

    it('sets generic error on failed login without message', async () => {
      const errorResponse = {
        response: {},
      };

      (api.login as any).mockRejectedValue(errorResponse);

      const store = useAuthStore.getState();

      try {
        await store.login('test@example.com', 'wrongpassword');
      } catch (error) {
        // Expected to throw
      }

      const state = useAuthStore.getState();
      expect(state.error).toBe('Login fallito');
    });

    it('sets isLoading during login', async () => {
      const mockUser = {
        id: 1,
        email: 'test@example.com',
        full_name: 'Test User',
        role: 'user' as const,
        created_at: '2024-01-01',
      };

      (api.login as any).mockImplementation(() => {
        // Check loading state during async operation
        const state = useAuthStore.getState();
        expect(state.isLoading).toBe(true);
        return Promise.resolve({ access_token: 'token123', refresh_token: 'refresh123' });
      });
      (api.getCurrentUser as any).mockResolvedValue(mockUser);

      const store = useAuthStore.getState();
      await store.login('test@example.com', 'password123');
    });
  });

  describe('register', () => {
    it('registers user and sets authenticated state', async () => {
      const mockUser = {
        id: 1,
        email: 'new@example.com',
        full_name: 'New User',
        role: 'user' as const,
        created_at: '2024-01-01',
      };

      (api.register as any).mockResolvedValue({ access_token: 'token456', refresh_token: 'refresh456' });
      (api.getCurrentUser as any).mockResolvedValue(mockUser);

      const store = useAuthStore.getState();
      await store.register('new@example.com', 'password123', 'New User');

      const state = useAuthStore.getState();
      expect(state.user).toEqual(mockUser);
      expect(state.isAuthenticated).toBe(true);
      expect(state.error).toBeNull();
    });

    it('sets error on failed registration', async () => {
      const errorResponse = {
        response: {
          data: {
            message: 'Email already exists',
          },
        },
      };

      (api.register as any).mockRejectedValue(errorResponse);

      const store = useAuthStore.getState();

      try {
        await store.register('existing@example.com', 'password123', 'User');
      } catch (error) {
        // Expected to throw
      }

      const state = useAuthStore.getState();
      expect(state.error).toBe('Email already exists');
    });
  });

  describe('logout', () => {
    it('clears user state and calls API logout', async () => {
      // Set initial state
      useAuthStore.setState({
        user: {
          id: 1,
          email: 'test@example.com',
          full_name: 'Test User',
          role: 'user',
          created_at: '2024-01-01',
        },
        isAuthenticated: true,
      });

      (api.logout as any).mockResolvedValue({});

      const store = useAuthStore.getState();
      await store.logout();

      const state = useAuthStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(api.logout).toHaveBeenCalled();
    });

    it('clears state even if API logout fails', async () => {
      useAuthStore.setState({
        user: {
          id: 1,
          email: 'test@example.com',
          full_name: 'Test User',
          role: 'user',
          created_at: '2024-01-01',
        },
        isAuthenticated: true,
      });

      (api.logout as any).mockRejectedValue(new Error('Network error'));

      const store = useAuthStore.getState();
      await store.logout();

      // Should still clear state
      const state = useAuthStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
    });
  });

  describe('fetchCurrentUser', () => {
    it('returns early if no token in localStorage', async () => {
      // Ensure no token
      localStorage.removeItem('access_token');

      const store = useAuthStore.getState();
      await store.fetchCurrentUser();

      const state = useAuthStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      expect(api.getCurrentUser).not.toHaveBeenCalled();
    });

    it('fetches and sets current user when token exists', async () => {
      const mockUser = {
        id: 1,
        email: 'test@example.com',
        full_name: 'Test User',
        role: 'user' as const,
        created_at: '2024-01-01',
      };

      // Set token in localStorage
      localStorage.setItem('access_token', 'valid_token');

      (api.getCurrentUser as any).mockResolvedValue(mockUser);

      const store = useAuthStore.getState();
      await store.fetchCurrentUser();

      const state = useAuthStore.getState();
      expect(state.user).toEqual(mockUser);
      expect(state.isAuthenticated).toBe(true);
    });

    it('clears state on fetch failure', async () => {
      // Set tokens in localStorage
      localStorage.setItem('access_token', 'invalid_token');
      localStorage.setItem('refresh_token', 'refresh_token');

      (api.getCurrentUser as any).mockRejectedValue(new Error('Unauthorized'));

      const store = useAuthStore.getState();
      await store.fetchCurrentUser();

      const state = useAuthStore.getState();
      expect(state.user).toBeNull();
      expect(state.isAuthenticated).toBe(false);
      // Tokens should be removed
      expect(localStorage.getItem('access_token')).toBeNull();
      expect(localStorage.getItem('refresh_token')).toBeNull();
    });
  });

  describe('clearError', () => {
    it('clears error state', () => {
      useAuthStore.setState({ error: 'Some error' });

      const store = useAuthStore.getState();
      store.clearError();

      const state = useAuthStore.getState();
      expect(state.error).toBeNull();
    });
  });
});
