import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormDescription,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { useToast } from '@/hooks/use-toast';

const workspaceSchema = z.object({
  name: z
    .string()
    .min(3, 'Nome deve essere almeno 3 caratteri')
    .max(50, 'Nome non può superare 50 caratteri')
    .regex(/^[a-zA-Z0-9\s]+$/, 'Solo caratteri alfanumerici e spazi'),
  description: z
    .string()
    .max(200, 'Descrizione non può superare 200 caratteri')
    .optional(),
});

type WorkspaceFormData = z.infer<typeof workspaceSchema>;

interface WorkspaceCreateModalProps {
  open: boolean;
  onClose: () => void;
  onWorkspaceCreated: (workspace: any) => void;
}

export function WorkspaceCreateModal({
  open,
  onClose,
  onWorkspaceCreated,
}: WorkspaceCreateModalProps) {
  const [isSubmitting, setIsSubmitting] = useState(false);
  const { toast } = useToast();

  const form = useForm<WorkspaceFormData>({
    resolver: zodResolver(workspaceSchema),
    defaultValues: {
      name: '',
      description: '',
    },
  });

  const onSubmit = async (data: WorkspaceFormData) => {
    setIsSubmitting(true);

    try {
      const response = await fetch('/api/workspaces', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
        credentials: 'include',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Creazione workspace fallita');
      }

      const workspace = await response.json();

      toast({
        title: 'Workspace creato',
        description: `Workspace "${workspace.name}" creato con successo`,
      });

      onWorkspaceCreated(workspace);
      form.reset();
      onClose();
    } catch (error) {
      toast({
        title: 'Errore',
        description:
          error instanceof Error
            ? error.message
            : 'Impossibile creare il workspace',
        variant: 'destructive',
      });
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[525px]">
        <DialogHeader>
          <DialogTitle>Crea nuovo workspace</DialogTitle>
          <DialogDescription>
            Crea un workspace per organizzare i tuoi knowledge base e collaborare
            con il team.
          </DialogDescription>
        </DialogHeader>

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-6">
            <FormField
              control={form.control}
              name="name"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Nome workspace *</FormLabel>
                  <FormControl>
                    <Input
                      placeholder="es. Contratti Clienti 2024"
                      {...field}
                      disabled={isSubmitting}
                      aria-label="Nome workspace"
                    />
                  </FormControl>
                  <FormDescription>
                    3-50 caratteri, solo alfanumerici e spazi
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="description"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Descrizione (opzionale)</FormLabel>
                  <FormControl>
                    <Textarea
                      placeholder="Descrivi lo scopo di questo workspace..."
                      className="resize-none"
                      rows={3}
                      {...field}
                      disabled={isSubmitting}
                      aria-label="Descrizione workspace"
                    />
                  </FormControl>
                  <FormDescription>
                    Massimo 200 caratteri
                  </FormDescription>
                  <FormMessage />
                </FormItem>
              )}
            />

            <DialogFooter>
              <Button
                type="button"
                variant="outline"
                onClick={onClose}
                disabled={isSubmitting}
              >
                Annulla
              </Button>
              <Button type="submit" disabled={isSubmitting}>
                {isSubmitting ? 'Creazione...' : 'Crea workspace'}
              </Button>
            </DialogFooter>
          </form>
        </Form>
      </DialogContent>
    </Dialog>
  );
}
