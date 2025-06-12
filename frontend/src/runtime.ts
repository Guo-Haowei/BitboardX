import { Game } from './game';
import { Renderer } from './renderer';
import { EventManager } from './event-manager';
import { Display } from './display';

export interface RuntimeModule {
  getName(): string;
  init(): boolean;
  tick(): void;
}

class Runtime {
  public display: Display;
  public renderer: Renderer;
  public game: Game;
  public eventManager: EventManager;

  private modules: RuntimeModule[];

  public constructor() {
    this.display = new Display();
    this.renderer = new Renderer();
    this.game = new Game();
    this.eventManager = new EventManager();
    this.modules = [
      this.display,
      this.renderer,
      this.game,
      this.eventManager,
    ];
  }

  public addModule(module: RuntimeModule): void {
    this.modules.push(module);
  }

  public init(): boolean {
    for (const module of this.modules) {
      if (!module.init()) {
        // eslint-disable-next-line no-console
        console.error(`Failed to initialize '${module.getName()}`);
        return false;
      }
    }
    return true;
  }

  public tick(): void {
    for (const module of this.modules) {
      module.tick();
    }
  }
}

export const runtime = new Runtime();
