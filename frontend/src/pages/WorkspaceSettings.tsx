import { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useToast } from '@/hooks/use-toast';
import { Loader2, Users, Settings2, AlertTriangle, Trash2, UserPlus, UserMinus, Shield } from 'lucide-react';
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';

interface Workspace {
  id: string;
  name: string;
  description?: string;
  user_role: 'admin' | 'editor' | 'viewer';
  member_count: number;
  kb_count: number;
  created_at: string;
}

interface Member {
  user_id: number;
  name: string;
  email: string;
  role: 'admin' | 'editor' | 'viewer';
  joined_at: string;
}

export function WorkspaceSettings() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const { toast } = useToast();

  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [members, setMembers] = useState<Member[]>([]);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);

  // General settings
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  // Member management
  const [newMemberEmail, setNewMemberEmail] = useState('');
  const [newMemberRole, setNewMemberRole] = useState<'editor' | 'viewer'>('editor');
  const [addingMember, setAddingMember] = useState(false);

  useEffect(() => {
    loadWorkspace();
    loadMembers();
  }, [id]);

  const loadWorkspace = async () => {
    try {
      const response = await fetch(`/api/workspaces/${id}`, {
        credentials: 'include',
      });

      if (!response.ok) throw new Error('Caricamento workspace fallito');

      const data = await response.json();
      setWorkspace(data);
      setName(data.name);
      setDescription(data.description || '');
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile caricare workspace',
        variant: 'destructive',
      });
    } finally {
      setLoading(false);
    }
  };

  const loadMembers = async () => {
    try {
      const response = await fetch(`/api/workspaces/${id}/members`, {
        credentials: 'include',
      });

      if (!response.ok) throw new Error('Caricamento membri fallito');

      const data = await response.json();
      setMembers(data.members || []);
    } catch (error) {
      console.error('Failed to load members:', error);
    }
  };

  const handleSaveGeneral = async () => {
    setSaving(true);

    try {
      const response = await fetch(`/api/workspaces/${id}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ name, description }),
      });

      if (!response.ok) throw new Error('Salvataggio fallito');

      toast({
        title: 'Salvato',
        description: 'Impostazioni workspace aggiornate',
      });

      loadWorkspace();
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile salvare le modifiche',
        variant: 'destructive',
      });
    } finally {
      setSaving(false);
    }
  };

  const handleAddMember = async () => {
    if (!newMemberEmail.trim()) return;

    setAddingMember(true);

    try {
      const response = await fetch(`/api/workspaces/${id}/members`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          email: newMemberEmail,
          role: newMemberRole,
        }),
      });

      if (!response.ok) throw new Error('Aggiunta membro fallita');

      toast({
        title: 'Membro aggiunto',
        description: `${newMemberEmail} aggiunto come ${newMemberRole}`,
      });

      setNewMemberEmail('');
      loadMembers();
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile aggiungere membro',
        variant: 'destructive',
      });
    } finally {
      setAddingMember(false);
    }
  };

  const handleRemoveMember = async (userId: number) => {
    try {
      const response = await fetch(`/api/workspaces/${id}/members/${userId}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (!response.ok) throw new Error('Rimozione membro fallita');

      toast({
        title: 'Membro rimosso',
        description: 'Membro rimosso dal workspace',
      });

      loadMembers();
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile rimuovere membro',
        variant: 'destructive',
      });
    }
  };

  const handleChangeRole = async (userId: number, newRole: string) => {
    try {
      const response = await fetch(`/api/workspaces/${id}/members/${userId}`, {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({ role: newRole }),
      });

      if (!response.ok) throw new Error('Modifica ruolo fallita');

      toast({
        title: 'Ruolo modificato',
        description: `Ruolo aggiornato a ${newRole}`,
      });

      loadMembers();
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile modificare ruolo',
        variant: 'destructive',
      });
    }
  };

  const handleDeleteWorkspace = async () => {
    try {
      const response = await fetch(`/api/workspaces/${id}`, {
        method: 'DELETE',
        credentials: 'include',
      });

      if (!response.ok) throw new Error('Eliminazione workspace fallita');

      toast({
        title: 'Workspace eliminato',
        description: 'Workspace eliminato definitivamente',
      });

      navigate('/');
    } catch (error) {
      toast({
        title: 'Errore',
        description: 'Impossibile eliminare workspace',
        variant: 'destructive',
      });
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <Loader2 className="w-8 h-8 animate-spin text-primary-500" />
      </div>
    );
  }

  if (!workspace || workspace.user_role !== 'admin') {
    return (
      <div className="p-6">
        <Card>
          <CardHeader>
            <CardTitle>Accesso negato</CardTitle>
            <CardDescription>
              Solo gli amministratori possono accedere alle impostazioni workspace
            </CardDescription>
          </CardHeader>
          <CardFooter>
            <Button onClick={() => navigate('/')}>Torna alla home</Button>
          </CardFooter>
        </Card>
      </div>
    );
  }

  return (
    <div className="max-w-5xl mx-auto p-6 space-y-6">
      <div>
        <h1 className="text-2xl font-bold text-white">Impostazioni Workspace</h1>
        <p className="text-gray-400 mt-1">{workspace.name}</p>
      </div>

      <Tabs defaultValue="general" className="space-y-6">
        <TabsList>
          <TabsTrigger value="general">
            <Settings2 className="w-4 h-4 mr-2" />
            Generale
          </TabsTrigger>
          <TabsTrigger value="members">
            <Users className="w-4 h-4 mr-2" />
            Membri ({members.length})
          </TabsTrigger>
          <TabsTrigger value="danger">
            <AlertTriangle className="w-4 h-4 mr-2" />
            Zona Pericolosa
          </TabsTrigger>
        </TabsList>

        {/* General Tab */}
        <TabsContent value="general">
          <Card>
            <CardHeader>
              <CardTitle>Informazioni Generali</CardTitle>
              <CardDescription>
                Modifica nome e descrizione del workspace
              </CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="name">Nome workspace *</Label>
                <Input
                  id="name"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Nome workspace"
                />
              </div>

              <div className="space-y-2">
                <Label htmlFor="description">Descrizione</Label>
                <Input
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder="Descrizione opzionale"
                />
              </div>
            </CardContent>
            <CardFooter>
              <Button onClick={handleSaveGeneral} disabled={saving}>
                {saving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
                Salva modifiche
              </Button>
            </CardFooter>
          </Card>
        </TabsContent>

        {/* Members Tab */}
        <TabsContent value="members" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Aggiungi Membro</CardTitle>
              <CardDescription>Invita nuovi membri al workspace</CardDescription>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="flex gap-2">
                <Input
                  type="email"
                  placeholder="Email utente"
                  value={newMemberEmail}
                  onChange={(e) => setNewMemberEmail(e.target.value)}
                  className="flex-1"
                />
                <Select
                  value={newMemberRole}
                  onValueChange={(value: 'editor' | 'viewer') =>
                    setNewMemberRole(value)
                  }
                >
                  <SelectTrigger className="w-32">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="editor">Editor</SelectItem>
                    <SelectItem value="viewer">Viewer</SelectItem>
                  </SelectContent>
                </Select>
                <Button onClick={handleAddMember} disabled={addingMember}>
                  {addingMember ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <UserPlus className="h-4 w-4" />
                  )}
                </Button>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>Membri ({members.length})</CardTitle>
              <CardDescription>Gestisci membri e ruoli</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-2">
                {members.map((member) => (
                  <div
                    key={member.user_id}
                    className="flex items-center justify-between p-3 rounded-lg bg-dark-50 hover:bg-dark-100 transition-colors"
                  >
                    <div className="flex-1">
                      <p className="text-sm font-medium text-white">
                        {member.name}
                      </p>
                      <p className="text-xs text-gray-400">{member.email}</p>
                    </div>

                    <div className="flex items-center gap-2">
                      <Select
                        value={member.role}
                        onValueChange={(value) =>
                          handleChangeRole(member.user_id, value)
                        }
                      >
                        <SelectTrigger className="w-28">
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="admin">
                            <div className="flex items-center gap-2">
                              <Shield className="h-3 w-3" />
                              Admin
                            </div>
                          </SelectItem>
                          <SelectItem value="editor">Editor</SelectItem>
                          <SelectItem value="viewer">Viewer</SelectItem>
                        </SelectContent>
                      </Select>

                      <Button
                        variant="ghost"
                        size="icon"
                        onClick={() => handleRemoveMember(member.user_id)}
                      >
                        <UserMinus className="h-4 w-4" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Danger Zone Tab */}
        <TabsContent value="danger">
          <Card className="border-destructive/50">
            <CardHeader>
              <CardTitle className="text-destructive">Zona Pericolosa</CardTitle>
              <CardDescription>
                Azioni irreversibili. Procedere con cautela.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div>
                  <h3 className="text-sm font-medium text-white mb-2">
                    Elimina Workspace
                  </h3>
                  <p className="text-sm text-gray-400 mb-4">
                    Questa azione eliminerà definitivamente il workspace, tutti i
                    knowledge base e i dati associati. Questa azione non può essere
                    annullata.
                  </p>
                  <Button
                    variant="destructive"
                    onClick={() => setDeleteDialogOpen(true)}
                  >
                    <Trash2 className="h-4 w-4 mr-2" />
                    Elimina workspace
                  </Button>
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      {/* Delete Confirmation Dialog */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Sei assolutamente sicuro?</AlertDialogTitle>
            <AlertDialogDescription>
              Questa azione non può essere annullata. Eliminerà definitivamente il
              workspace <strong>{workspace.name}</strong>, tutti i knowledge base
              ({workspace.kb_count}) e rimuoverà tutti i membri ({workspace.member_count}).
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Annulla</AlertDialogCancel>
            <AlertDialogAction
              onClick={handleDeleteWorkspace}
              className="bg-destructive hover:bg-destructive/90"
            >
              Elimina definitivamente
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
}
