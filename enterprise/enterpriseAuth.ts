// ═══════════════════════════════════════════════════════════════════════
//  enterpriseAuth.ts — Sistema de Autenticación Enterprise
//  QuantumEnergyOS — Gestor de usuarios, roles y autenticación segura
// ═══════════════════════════════════════════════════════════════════════

import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";

// ═══════════════════════════════════════════════════════════════════════
//  Tipos y Interfaces
// ═══════════════════════════════════════════════════════════════════════

export type UserRole = 
  | "admin" 
  | "manager" 
  | "operator" 
  | "viewer" 
  | "auditor";

export interface EnterpriseUser {
  id: string;
  email: string;
  name: string;
  role: UserRole;
  department: string;
  createdAt: string;
  lastLogin: string | null;
  isActive: boolean;
  twoFactorEnabled: boolean;
  avatar?: string;
}

export interface LoginCredentials {
  email: string;
  password: string;
  rememberMe?: boolean;
}

export interface AuthState {
  isAuthenticated: boolean;
  user: EnterpriseUser | null;
  token: string | null;
  loading: boolean;
  error: string | null;
}

// ═══════════════════════════════════════════════════════════════════════
//  Configuración de Roles y Permisos
// ═══════════════════════════════════════════════════════════════════════

export const ROLE_PERMISSIONS: Record<UserRole, string[]> = {
  admin: [
    "users:read", "users:write", "users:delete",
    "roles:read", "roles:write",
    "audit:read", "audit:write", "audit:delete",
    "reports:read", "reports:write", "reports:export",
    "backup:read", "backup:write", "backup:restore",
    "settings:read", "settings:write",
    "grid:read", "grid:write", "grid:execute",
    "quartz:read", "quartz:write", "quartz:execute",
    "solar:read", "solar:write",
  ],
  manager: [
    "users:read",
    "audit:read",
    "reports:read", "reports:write", "reports:export",
    "grid:read", "grid:write", "grid:execute",
    "quartz:read", "quartz:execute",
    "solar:read",
  ],
  operator: [
    "grid:read", "grid:write", "grid:execute",
    "quartz:read", "quartz:execute",
    "solar:read",
  ],
  viewer: [
    "grid:read",
    "quartz:read",
    "solar:read",
  ],
  auditor: [
    "audit:read",
    "reports:read", "reports:export",
  ],
};

export const ROLE_LABELS: Record<UserRole, string> = {
  admin: "Administrador",
  manager: "Gerente",
  operator: "Operador",
  viewer: "Visor",
  auditor: "Auditor",
};

// ═══════════════════════════════════════════════════════════════════════
//  Store de Autenticación Enterprise
// ═══════════════════════════════════════════════════════════════════════

interface AuthStore extends AuthState {
  login: (credentials: LoginCredentials) => Promise<boolean>;
  logout: () => void;
  registerUser: (user: Omit<EnterpriseUser, "id" | "createdAt" | "lastLogin">) => Promise<boolean>;
  updateUser: (userId: string, updates: Partial<EnterpriseUser>) => Promise<boolean>;
  deleteUser: (userId: string) => Promise<boolean>;
  hasPermission: (permission: string) => boolean;
  getAllUsers: () => EnterpriseUser[];
  setError: (error: string | null) => void;
}

// ═══════════════════════════════════════════════════════════════════════
//  Utilidades de Seguridad
// ═══════════════════════════════════════════════════════════════════════

function generateUserId(): string {
  return `user_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
}

function generateToken(): string {
  const array = new Uint8Array(32);
  crypto.getRandomValues(array);
  return Array.from(array, byte => byte.toString(16).padStart(2, "0")).join("");
}

function hashPassword(password: string): string {
  const encoder = new TextEncoder();
  const data = encoder.encode(password);
  return crypto.subtle.digest("SHA-256", data).then(buffer => {
    const hashArray = Array.from(new Uint8Array(buffer));
    return hashArray.map(b => b.toString(16).padStart(2, "0")).join("");
  });
}

async function verifyPassword(password: string, storedHash: string): Promise<boolean> {
  const hash = await hashPassword(password);
  return hash === storedHash;
}

// ═══════════════════════════════════════════════════════════════════════
//  Usuarios Demo (Seed Data)
// ═══════════════════════════════════════════════════════════════════════

const DEMO_USERS: EnterpriseUser[] = [
  {
    id: "user_admin_001",
    email: "admin@quantumenergy.os",
    name: "Gio Corpus",
    role: "admin",
    department: "IT",
    createdAt: "2024-01-15T08:00:00Z",
    lastLogin: "2026-04-03T10:30:00Z",
    isActive: true,
    twoFactorEnabled: true,
    avatar: undefined,
  },
  {
    id: "user_manager_001",
    email: "manager@quantumenergy.os",
    name: "Eliot Hernandez",
    role: "manager",
    department: "Operaciones",
    createdAt: "2024-03-20T09:00:00Z",
    lastLogin: "2026-04-02T14:22:00Z",
    isActive: true,
    twoFactorEnabled: false,
    avatar: undefined,
  },
  {
    id: "user_operator_001",
    email: "operator@quantumenergy.os",
    name: "Carlos Mendez",
    role: "operator",
    department: "Ingeniería",
    createdAt: "2024-06-10T10:30:00Z",
    lastLogin: "2026-04-03T08:15:00Z",
    isActive: true,
    twoFactorEnabled: false,
    avatar: undefined,
  },
  {
    id: "user_auditor_001",
    email: "auditor@quantumenergy.os",
    name: "Ana Laura",
    role: "auditor",
    department: "Cumplimiento",
    createdAt: "2024-09-05T11:00:00Z",
    lastLogin: "2026-04-01T16:45:00Z",
    isActive: true,
    twoFactorEnabled: true,
    avatar: undefined,
  },
];

// ═══════════════════════════════════════════════════════════════════════
//  Implementación del Store
// ═══════════════════════════════════════════════════════════════════════

export const useEnterpriseAuth = create<AuthStore>()(
  persist(
    (set, get) => ({
      isAuthenticated: false,
      user: null,
      token: null,
      loading: false,
      error: null,

      login: async (credentials: LoginCredentials): Promise<boolean> => {
        set({ loading: true, error: null });
        
        try {
          // Simular delay de red
          await new Promise(resolve => setTimeout(resolve, 800));
          
          // Buscar usuario por email
          const users = get().getAllUsers();
          const foundUser = users.find(u => u.email === credentials.email);
          
          if (!foundUser) {
            set({ error: "Usuario no encontrado", loading: false });
            return false;
          }
          
          if (!foundUser.isActive) {
            set({ error: "Cuenta desactivada. Contacte al administrador.", loading: false });
            return false;
          }
          
          // En modo demo, cualquier password funciona para usuarios demo
          // En producción, verificar hash de contraseña
          const token = generateToken();
          
          const updatedUser = {
            ...foundUser,
            lastLogin: new Date().toISOString(),
          };
          
          set({
            isAuthenticated: true,
            user: updatedUser,
            token,
            loading: false,
            error: null,
          });
          
          return true;
        } catch (err) {
          set({ error: "Error de autenticación", loading: false });
          return false;
        }
      },

      logout: () => {
        set({
          isAuthenticated: false,
          user: null,
          token: null,
          error: null,
        });
      },

      registerUser: async (userData): Promise<boolean> => {
        set({ loading: true, error: null });
        
        try {
          await new Promise(resolve => setTimeout(resolve, 500));
          
          const users = get().getAllUsers();
          
          // Verificar email único
          if (users.some(u => u.email === userData.email)) {
            set({ error: "El email ya está registrado", loading: false });
            return false;
          }
          
          const newUser: EnterpriseUser = {
            ...userData,
            id: generateUserId(),
            createdAt: new Date().toISOString(),
            lastLogin: null,
          };
          
          // Guardar en localStorage (en producción sería API)
          const storedUsers = JSON.parse(localStorage.getItem("enterprise_users") || "[]");
          storedUsers.push(newUser);
          localStorage.setItem("enterprise_users", JSON.stringify(storedUsers));
          
          set({ loading: false });
          return true;
        } catch (err) {
          set({ error: "Error al crear usuario", loading: false });
          return false;
        }
      },

      updateUser: async (userId: string, updates: Partial<EnterpriseUser>): Promise<boolean> => {
        set({ loading: true, error: null });
        
        try {
          await new Promise(resolve => setTimeout(resolve, 300));
          
          const storedUsers = JSON.parse(localStorage.getItem("enterprise_users") || "[]");
          const index = storedUsers.findIndex((u: EnterpriseUser) => u.id === userId);
          
          if (index === -1) {
            set({ error: "Usuario no encontrado", loading: false });
            return false;
          }
          
          storedUsers[index] = { ...storedUsers[index], ...updates };
          localStorage.setItem("enterprise_users", JSON.stringify(storedUsers));
          
          // Actualizar usuario actual si es el mismo
          const currentUser = get().user;
          if (currentUser?.id === userId) {
            set({ user: { ...currentUser, ...updates } });
          }
          
          set({ loading: false });
          return true;
        } catch (err) {
          set({ error: "Error al actualizar usuario", loading: false });
          return false;
        }
      },

      deleteUser: async (userId: string): Promise<boolean> => {
        set({ loading: true, error: null });
        
        try {
          await new Promise(resolve => setTimeout(resolve, 300));
          
          const storedUsers = JSON.parse(localStorage.getItem("enterprise_users") || "[]");
          const filtered = storedUsers.filter((u: EnterpriseUser) => u.id !== userId);
          localStorage.setItem("enterprise_users", JSON.stringify(filtered));
          
          // Si el usuario actual es el mismo, hacer logout
          const currentUser = get().user;
          if (currentUser?.id === userId) {
            get().logout();
          }
          
          set({ loading: false });
          return true;
        } catch (err) {
          set({ error: "Error al eliminar usuario", loading: false });
          return false;
        }
      },

      hasPermission: (permission: string): boolean => {
        const user = get().user;
        if (!user) return false;
        
        const permissions = ROLE_PERMISSIONS[user.role] || [];
        return permissions.includes(permission);
      },

      getAllUsers: (): EnterpriseUser[] => {
        // Combinar usuarios demo con usuarios personalizados
        const storedUsers = JSON.parse(localStorage.getItem("enterprise_users") || "[]");
        const allUsers = [...DEMO_USERS, ...storedUsers];
        
        // Deduplicar por ID
        const uniqueUsers = allUsers.reduce((acc: EnterpriseUser[], user) => {
          if (!acc.find(u => u.id === user.id)) {
            acc.push(user);
          }
          return acc;
        }, []);
        
        return uniqueUsers;
      },

      setError: (error: string | null) => set({ error }),
    }),
    {
      name: "enterprise-auth-storage",
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        isAuthenticated: state.isAuthenticated,
        user: state.user,
        token: state.token,
      }),
    }
  )
);

// ═══════════════════════════════════════════════════════════════════════
//  Hooks de Autenticación
// ═══════════════════════════════════════════════════════════════════════

export function useAuth() {
  const { isAuthenticated, user, token, logout } = useEnterpriseAuth();
  return { isAuthenticated, user, token, logout };
}

export function usePermissions() {
  const { user, hasPermission } = useEnterpriseAuth();
  return {
    userRole: user?.role || null,
    hasPermission,
    can: (permission: string) => hasPermission(permission),
  };
}