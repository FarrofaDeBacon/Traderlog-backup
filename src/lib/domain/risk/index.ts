/**
 * Domínio Central de Gerenciamento de Risco (Risk Domain)
 * 
 * Contém a modelagem rica e as lógicas de negócio puras (sem side-effects).
 * Módulos encapsulados fornecem a infraestrutura imutável pro resto da aplicação usar:
 *  - Controle de perdas financeiras (`risk-engine`)
 *  - Controle de regressão/metas de plano (`growth-engine`)
 *  - Controle comportamental (`discipline-engine`)
 */

export * from './types';
export * from './risk-engine';
export * from './growth-engine';
export * from './discipline-engine';
