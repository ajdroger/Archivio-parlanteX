import type { ReactNode } from 'react';
import { NavLink } from 'react-router-dom';
import { useAuthStore } from '../../store/authStore';
import { useAppStore } from '../../store/appStore';
import {
  MessageSquare,
  FileText,
  GitCompare,
  BarChart3,
  LogOut,
  Settings,
} from 'lucide-react';

interface MainLayoutProps {
  children: ReactNode;
}

export default function MainLayout({ children }: MainLayoutProps) {
  const { user, logout } = useAuthStore();
  const { currentKb } = useAppStore();

  const navigation = [
    { name: 'Chat', href: '/', icon: MessageSquare },
    { name: 'Documenti', href: '/documents', icon: FileText },
    { name: 'Confronto', href: '/compare', icon: GitCompare },
    { name: 'Analisi', href: '/analytics', icon: BarChart3 },
  ];

  return (
    <div className="min-h-screen bg-dark flex">
      {/* Sidebar */}
      <aside className="w-64 bg-dark-100 border-r border-dark-50 flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-dark-50">
          <h1 className="text-xl font-bold text-primary">
            🏛️ Archivio Parlante
          </h1>
          {currentKb && (
            <p className="text-xs text-gray-400 mt-1 truncate">
              KB: {currentKb.name}
            </p>
          )}
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-4 space-y-1">
          {navigation.map((item) => (
            <NavLink
              key={item.name}
              to={item.href}
              className={({ isActive }) =>
                `flex items-center gap-3 px-3 py-2 rounded-lg transition-colors ${
                  isActive
                    ? 'bg-primary/20 text-primary'
                    : 'text-gray-400 hover:bg-dark-50 hover:text-gray-300'
                }`
              }
            >
              <item.icon className="w-5 h-5" />
              <span className="font-medium">{item.name}</span>
            </NavLink>
          ))}
        </nav>

        {/* User Section */}
        <div className="p-4 border-t border-dark-50 space-y-2">
          {user?.role === 'admin' && (
            <NavLink
              to="/admin"
              className="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:bg-dark-50 hover:text-gray-300 transition-colors"
            >
              <Settings className="w-5 h-5" />
              <span className="font-medium">Admin</span>
            </NavLink>
          )}

          <div className="px-3 py-2 text-sm">
            <p className="font-medium text-gray-300 truncate">
              {user?.full_name}
            </p>
            <p className="text-xs text-gray-500 truncate">{user?.email}</p>
          </div>

          <button
            onClick={logout}
            className="flex items-center gap-3 px-3 py-2 rounded-lg text-gray-400 hover:bg-red-900/20 hover:text-red-400 transition-colors w-full"
          >
            <LogOut className="w-5 h-5" />
            <span className="font-medium">Esci</span>
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 overflow-auto">{children}</main>
    </div>
  );
}
