'use client';

import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { e2Client, E2User, E2Wallet } from './client';

interface AuthContextType {
  user: E2User | null;
  wallets: E2Wallet[];
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
  login: (email: string, password: string) => Promise<void>;
  signup: (email: string, password: string, firstName: string, lastName: string) => Promise<void>;
  logout: () => Promise<void>;
  refreshUser: () => Promise<void>;
  refreshWallets: () => Promise<void>;
  clearError: () => void;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

interface AuthProviderProps {
  children: ReactNode;
}

/**
 * Authentication Provider
 * Manages user authentication state, JWT tokens, and wallet information
 */
export function AuthProvider({ children }: AuthProviderProps) {
  const [user, setUser] = useState<E2User | null>(null);
  const [wallets, setWallets] = useState<E2Wallet[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  /**
   * Load user and wallets on mount if token exists
   */
  useEffect(() => {
    const initializeAuth = async () => {
      setIsLoading(true);
      setError(null);

      try {
        if (e2Client.isAuthenticated()) {
          // Fetch user data
          const userData = await e2Client.getCurrentUser();
          setUser(userData);

          // Fetch wallets
          try {
            const walletsData = await e2Client.getWallets();
            setWallets(walletsData);
          } catch (walletError) {
            console.warn('Failed to load wallets:', walletError);
            // Don't fail auth if wallets can't be loaded
          }
        }
      } catch (err: any) {
        console.error('Failed to initialize auth:', err);
        // Token is invalid, clear it
        e2Client.logout();
        setUser(null);
        setWallets([]);
      } finally {
        setIsLoading(false);
      }
    };

    initializeAuth();
  }, []);

  /**
   * Login with email and password
   */
  const login = async (email: string, password: string) => {
    setIsLoading(true);
    setError(null);

    try {
      // Authenticate
      const authResponse = await e2Client.login(email, password);

      // Fetch full user data
      const userData = await e2Client.getCurrentUser();
      setUser(userData);

      // Fetch wallets
      try {
        const walletsData = await e2Client.getWallets();
        setWallets(walletsData);
      } catch (walletError) {
        console.warn('Failed to load wallets after login:', walletError);
      }
    } catch (err: any) {
      const errorMessage = err.message || 'Login failed';
      setError(errorMessage);
      throw err; // Re-throw so component can handle it
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Sign up new user
   */
  const signup = async (
    email: string,
    password: string,
    firstName: string,
    lastName: string
  ) => {
    setIsLoading(true);
    setError(null);

    try {
      // Create account
      const authResponse = await e2Client.signup({
        email,
        password,
        first_name: firstName,
        last_name: lastName,
      });

      // Fetch full user data
      const userData = await e2Client.getCurrentUser();
      setUser(userData);

      // Fetch wallets (should have one created automatically)
      try {
        const walletsData = await e2Client.getWallets();
        setWallets(walletsData);
      } catch (walletError) {
        console.warn('Failed to load wallets after signup:', walletError);
      }
    } catch (err: any) {
      const errorMessage = err.message || 'Signup failed';
      setError(errorMessage);
      throw err;
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Logout current user
   */
  const logout = async () => {
    setIsLoading(true);

    try {
      await e2Client.logout();
      setUser(null);
      setWallets([]);
      setError(null);
    } catch (err: any) {
      console.error('Logout error:', err);
      // Clear local state anyway
      setUser(null);
      setWallets([]);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Refresh user data
   */
  const refreshUser = async () => {
    if (!e2Client.isAuthenticated()) {
      return;
    }

    try {
      const userData = await e2Client.getCurrentUser();
      setUser(userData);
    } catch (err: any) {
      console.error('Failed to refresh user:', err);
      // Token might be invalid
      if (err.message?.includes('401') || err.message?.includes('Unauthorized')) {
        setUser(null);
        setWallets([]);
      }
    }
  };

  /**
   * Refresh wallets data
   */
  const refreshWallets = async () => {
    if (!e2Client.isAuthenticated()) {
      return;
    }

    try {
      const walletsData = await e2Client.getWallets();
      setWallets(walletsData);
    } catch (err: any) {
      console.error('Failed to refresh wallets:', err);
    }
  };

  /**
   * Clear error message
   */
  const clearError = () => {
    setError(null);
  };

  const value: AuthContextType = {
    user,
    wallets,
    isAuthenticated: user !== null,
    isLoading,
    error,
    login,
    signup,
    logout,
    refreshUser,
    refreshWallets,
    clearError,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

/**
 * Hook to use authentication context
 */
export function useAuth() {
  const context = useContext(AuthContext);

  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }

  return context;
}

/**
 * Hook to require authentication
 * Redirects to login if not authenticated
 */
export function useRequireAuth() {
  const auth = useAuth();

  useEffect(() => {
    if (!auth.isLoading && !auth.isAuthenticated) {
      // Redirect to login
      if (typeof window !== 'undefined') {
        window.location.href = '/login';
      }
    }
  }, [auth.isLoading, auth.isAuthenticated]);

  return auth;
}
