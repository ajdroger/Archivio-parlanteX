// ============================================================================
// Archivio Parlante — Workspace Switcher Component
// ============================================================================
// Fase 6.3.5: Dropdown to select current workspace + filter KBs

import React, { useEffect, useState } from 'react';
import { ChevronDown, Users, Plus, Settings } from 'lucide-react';
import { useAppStore } from '../../store/appStore';
import api from '../../lib/api';

interface Workspace {
  id: string;
  name: string;
  user_role: 'admin' | 'member' | 'viewer';
  member_count: number;
  kb_count: number;
}

export const WorkspaceSwitcher: React.FC = () => {
  const [workspaces, setWorkspaces] = useState<Workspace[]>([]);
  const [isOpen, setIsOpen] = useState(false);
  const [loading, setLoading] = useState(false);

  const { currentWorkspace, setCurrentWorkspace } = useAppStore();

  // Fetch user's workspaces on mount
  useEffect(() => {
    fetchWorkspaces();
  }, []);

  const fetchWorkspaces = async () => {
    setLoading(true);
    try {
      const response = await api.get('/workspaces');
      const data = response.data;

      setWorkspaces(data.workspaces || []);

      // Set default workspace if none selected
      if (!currentWorkspace && data.workspaces.length > 0) {
        setCurrentWorkspace(data.workspaces[0]);
      }
    } catch (error) {
      console.error('Failed to fetch workspaces:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleWorkspaceSelect = (workspace: Workspace) => {
    setCurrentWorkspace(workspace);
    setIsOpen(false);

    // Clear selected documents when switching workspace
    // (KB list will be filtered by workspace)
  };

  const handleCreateWorkspace = () => {
    // TODO Fase 6.3: Open modal to create new workspace
    console.log('Create workspace modal');
  };

  const handleManageWorkspace = () => {
    // TODO Fase 6.3: Navigate to workspace settings page
    console.log('Manage workspace');
  };

  if (loading && workspaces.length === 0) {
    return (
      <div className="animate-pulse flex items-center gap-2 px-3 py-2 bg-dark-200 rounded-md">
        <div className="w-4 h-4 bg-dark-100 rounded"></div>
        <div className="w-32 h-4 bg-dark-100 rounded"></div>
      </div>
    );
  }

  return (
    <div className="relative">
      {/* Current Workspace Button */}
      <button
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-2 bg-dark-200 hover:bg-dark-100 rounded-md transition-colors min-w-[200px] justify-between"
        aria-label="Select workspace"
        aria-expanded={isOpen}
      >
        <div className="flex items-center gap-2 flex-1 min-w-0">
          <Users className="w-4 h-4 text-primary-500 flex-shrink-0" />
          <span className="text-sm font-medium text-white truncate">
            {currentWorkspace?.name || 'Select workspace'}
          </span>
        </div>
        <ChevronDown
          className={`w-4 h-4 text-gray-400 transition-transform flex-shrink-0 ${
            isOpen ? 'rotate-180' : ''
          }`}
        />
      </button>

      {/* Dropdown Menu */}
      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
            aria-hidden="true"
          />

          {/* Menu */}
          <div className="absolute top-full left-0 mt-2 w-full min-w-[280px] bg-dark-200 border border-dark-100 rounded-md shadow-lg z-20 max-h-96 overflow-y-auto">
            {/* Workspace List */}
            <div className="py-1">
              {workspaces.map((workspace) => (
                <button
                  key={workspace.id}
                  onClick={() => handleWorkspaceSelect(workspace)}
                  className={`w-full text-left px-4 py-2 hover:bg-dark-100 transition-colors flex items-start gap-3 ${
                    currentWorkspace?.id === workspace.id
                      ? 'bg-dark-100'
                      : ''
                  }`}
                >
                  <Users className={`w-4 h-4 mt-0.5 flex-shrink-0 ${
                    currentWorkspace?.id === workspace.id
                      ? 'text-primary-500'
                      : 'text-gray-400'
                  }`} />

                  <div className="flex-1 min-w-0">
                    <div className="flex items-center justify-between gap-2">
                      <span className={`text-sm font-medium truncate ${
                        currentWorkspace?.id === workspace.id
                          ? 'text-primary-500'
                          : 'text-white'
                      }`}>
                        {workspace.name}
                      </span>

                      {workspace.user_role === 'admin' && (
                        <span className="text-xs px-1.5 py-0.5 bg-primary-500/20 text-primary-500 rounded flex-shrink-0">
                          Admin
                        </span>
                      )}
                    </div>

                    <div className="flex items-center gap-3 mt-1 text-xs text-gray-400">
                      <span>{workspace.member_count} members</span>
                      <span>•</span>
                      <span>{workspace.kb_count} KBs</span>
                    </div>
                  </div>
                </button>
              ))}
            </div>

            {/* Divider */}
            <div className="border-t border-dark-100 my-1"></div>

            {/* Actions */}
            <div className="py-1">
              <button
                onClick={handleCreateWorkspace}
                className="w-full text-left px-4 py-2 hover:bg-dark-100 transition-colors flex items-center gap-3 text-sm text-white"
              >
                <Plus className="w-4 h-4 text-primary-500" />
                Create workspace
              </button>

              {currentWorkspace && currentWorkspace.user_role === 'admin' && (
                <button
                  onClick={handleManageWorkspace}
                  className="w-full text-left px-4 py-2 hover:bg-dark-100 transition-colors flex items-center gap-3 text-sm text-white"
                >
                  <Settings className="w-4 h-4 text-gray-400" />
                  Manage workspace
                </button>
              )}
            </div>
          </div>
        </>
      )}
    </div>
  );
};
