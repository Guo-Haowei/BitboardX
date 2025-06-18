import { AnimationManager } from './animation-manager';
import { Display } from './display';
import { GameManager } from './game-manager';
import { MessageQueue } from './message-queue';
import { Renderer } from './renderer';

export interface RuntimeModule {
  init(): boolean;
  tick(): void;
}

class Runtime {
  public display: Display;
  public renderer: Renderer;
  public gameManager: GameManager;
  public messageQueue: MessageQueue;
  public animationManager: AnimationManager;

  private modules: RuntimeModule[];

  public constructor() {
    this.messageQueue = new MessageQueue();
    this.animationManager = new AnimationManager();
    this.display = new Display();
    this.renderer = new Renderer();
    this.gameManager = new GameManager();
    this.modules = [
      this.animationManager,
      this.display,
      this.renderer,
      this.gameManager,
      this.messageQueue,
    ];
  }

  public addModule(module: RuntimeModule): void {
    this.modules.push(module);
  }

  public init(): boolean {
    for (const module of this.modules) {
      if (!module.init()) {
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
