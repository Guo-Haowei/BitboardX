import { AnimationManager } from './animation-manager';
import { Display } from './display';
import { EventManager } from './event-manager';
import { GameManager } from './game-manager';
import { Renderer } from './renderer';

export interface RuntimeModule {
  getName(): string;
  init(): boolean;
  tick(): void;
}

class Runtime {
  public display: Display;
  public renderer: Renderer;
  public game: GameManager;
  public eventManager: EventManager;
  public animationManager: AnimationManager;

  private modules: RuntimeModule[];

  public constructor() {
    this.animationManager = new AnimationManager();
    this.display = new Display();
    this.renderer = new Renderer();
    this.game = new GameManager();
    this.eventManager = new EventManager();
    this.modules = [
      this.animationManager,
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
